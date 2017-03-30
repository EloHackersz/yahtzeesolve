/*
Copyright (c) 2015, Daniel Renninghoff
All rights reserved.

Redistribution and use in source and binary forms, with or without modification,
are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this
   list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

3. Neither the name of the copyright holder nor the names of its contributors
   may be used to endorse or promote products derived from this software without
   specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR
ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
(INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON
ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/

extern crate term;
extern crate rand;
extern crate byteorder;
extern crate yahtzeesolve;

//use std::env;
//use std::sync::mpsc;
use yahtzeesolve::LookupTable;
use yahtzeesolve::game::generators;
use yahtzeesolve::game::rules;
use yahtzeesolve::game::Game;
use rand::distributions::{IndependentSample, Range};
use std::io::Read;
use std::io::Write;
use std::net::{TcpListener};
use byteorder::{BigEndian, WriteBytesExt};

fn main() {
    //let args: Vec<String> = env::args().collect();
    let listener = TcpListener::bind("127.0.0.1:13337").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("got client");
                let mut buffer = [0; 1];
                loop {
                    match stream.read_exact(&mut buffer) {
                        Ok(_) => {
                            let skill : u8 = buffer[0];
                            println!("skill level is {:?}", skill);
                            let r = play(true, skill as u32) as u16;
                            println!("result = {:?}", r);
                            let mut wtr = vec![];
                            wtr.write_u16::<BigEndian>(r).unwrap();
                            stream.write(wtr.as_slice());
                        },
                        Err(_) => break
                    }
                }
            }
            Err(e) => {
                println!("network error ({:?}) please call Duncan", e);
            }
        }
    }

    /*
    match args.len() {
        1 => {
            println!("USAGE: {} [generate|play] <skill(0-100)>", &args[0]);
        }
        _ => {
            let mut skill : u32 = 100;
            if args.len() > 2 {
                skill = (&args[2]).parse::<u32>().unwrap();
            }
            match &args[1][..] {
                "generate" => {
                    let mut term = term::stdout().unwrap();
                    let (tx, rx) = mpsc::channel();
                    let thread = LookupTable::generate(tx);
                    for i in 0..100 {
                        term.carriage_return().unwrap();
                        write!(term, "Generating probabillity table... {}%", i).unwrap();
                        term.flush().unwrap();
                        rx.recv().unwrap();
                    }
                    let lookup = thread.join().unwrap();
                    lookup.write_to_file("probs.dat").unwrap();
                },
                "play" => {
                    play(false, skill);
                },
                "score" => {
                    play(true, skill);
                },
                _ => {
                }
            }
        }
    }
    */
}

fn play(silent : bool, skill : u32) -> u32 {
    let mut total_score = 0;
    let rollvec = generators::generate_dice_roll_possibilities();
    let dicekeeps = generators::generate_dice_keep_possibilities();
    let x = LookupTable::from_file("probs.dat").unwrap();
    let mut state = Game::new();
    for _ in 0..13 {
        let (cur_state, score) = calc_round(state, &x, &rollvec, &dicekeeps, silent, skill);
        state = cur_state;
        total_score += score
    }
    if !silent {
        println!("Total Score : {:?}", total_score);
    }
    total_score
}

fn mark_text(mark: u8) -> &'static str {
    let names = vec![
        "Ones",
        "Twos",
        "Threes",
        "Fours",
        "Fives", 
        "Sixes",
        "3 of a kind",
        "4 of a kind",
        "Full house",
        "Sm. straight",
        "Lg. straight",
        "YAHTZEE",
        "Chance"];
    names[(mark - 1) as usize]
}

fn roll_dices(amount: u8) -> u32 {
    match amount {
        0 => 0,
        _ => {
            let mut dices = 0;
            let mut rng = rand::thread_rng();
            let range = Range::new(1, 7);
            for i in 0..amount {
                let multiplier = 10u32.pow(i as u32);
                let thrown = range.ind_sample(&mut rng);
                dices += thrown * multiplier;
            }
            dices
        }
    }
}

fn calc_round(game: Game, lookup: &LookupTable, rollvec: &Vec<[u8; 6]>, dicekeeps: &Vec<[u8; 6]>, silent: bool, skill : u32) -> (Game, u32) {
    let (keep_1_states, keep_2_states) = yahtzeesolve::precalc_current_round(game, lookup, rollvec, dicekeeps, skill);
    let input: u32 = roll_dices(5);
    if !silent {
        println!("Thrown {:?}", input);
    }
    let inp1 = key_conv(input);
    let (_,kroll) = generators::gen_roll_prob(&inp1,&[0,0,0,0,0,0], &keep_1_states, skill);
    if !silent {
        println!("{:?}", kroll);
    }
    let nkept : u8 = kroll.iter().sum();
    let input2: u32 = roll_dices(5u8 - nkept);
    if !silent {
        println!("Thrown {:?}", input2);
    }
    let roll2 = key_conv(input2);
    let (_,kroll) = generators::gen_roll_prob(&roll2,&kroll, &keep_2_states, skill);
    if !silent {
        println!("{:?}", kroll);
    }
    let nkept2 : u8 = kroll.iter().sum();
    let input2: u32 = roll_dices(5u8 - nkept2);
    if !silent {
        println!("Thrown {:?}", input2);
    }
    let mut roll2 = key_conv(input2);
    roll2[0] += kroll[0];
    roll2[1] += kroll[1];
    roll2[2] += kroll[2];
    roll2[3] += kroll[3];
    roll2[4] += kroll[4];
    roll2[5] += kroll[5];
    let (_,choseni) = generators::choose_best_field(game, &roll2, lookup, skill);
    let marktxt = mark_text(choseni + 1);
    if !silent {
        println!("Mark  : {}", marktxt);
    }
    let scr = rules::score(&roll2, choseni);
    if !silent {
        println!("Score : {}", scr);
    }
    (game.next_turn(&roll2, choseni), scr as u32)
}

fn key_conv(input: u32) -> [u8;6] {
    let mut tmp = input;
    let mut out = [0u8; 6];
    while tmp != 0 {
        let tmp2 = (tmp % 10) as usize;
        if tmp2 > 0 && tmp2 <= 6 {
            out[tmp2 - 1] += 1;
        }
        tmp /= 10;
    }
    out
}