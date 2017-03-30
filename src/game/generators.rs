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

extern crate rand;

use std::collections::BTreeMap;
use game::Game;
use game::rules;
use game::lookuptable::LookupTable;
use game::generators::rand::distributions::IndependentSample;

pub fn make_failure_range() -> rand::distributions::Range<u32> {
    // [0,99]
    rand::distributions::Range::new(0u32, 100u32)
}

pub fn get_rand_success(rng : &mut rand::ThreadRng, range : &rand::distributions::Range<u32>, skill : u32) -> bool {
    range.ind_sample(rng) < skill
}

pub fn generate_dice_keep_possibilities() -> Vec<[u8; 6]> {
    let mut keeps = vec![];
    for ones in 0..6 {
        for twos in 0..(6-ones) {
            for threes in 0..(6-(ones+twos)) {
                for fours in 0..(6-(ones+twos+threes)) {
                    for fives in 0..(6-(ones+twos+threes+fours)) {
                        for sixes in 0..(6-(ones+twos+threes+fours+fives)) {
                            keeps.push([ones,twos,threes,fours,fives,sixes]);
                        }
                    }
                }
            }
        }
    }
    keeps
}

pub fn generate_dice_roll_possibilities() -> Vec<[u8; 6]> {
    let mut rolls = vec![];
    for ones in 0..6 {
        for twos in 0..(6-ones) {
            for threes in 0..(6-(ones+twos)) {
                for fours in 0..(6-(ones+twos+threes)) {
                    for fives in 0..(6-(ones+twos+threes+fours)) {
                        let sixes = 5 - (ones+twos+threes+fours+fives);
                        rolls.push([ones,twos,threes,fours,fives,sixes]);
                    }
                }
            }
        }
    }
    rolls
}

pub fn gen_start_prob(Game(state): Game, lookup: &LookupTable, rolls: &Vec<[u8; 6]>, dicekeeps: &Vec<[u8; 6]>) -> f64 {
    if state == 0b111_1111_1111_1111_1111 {
        return 35f64;
    }
    else if state > 0b111_1111_1111_1100_0000 {
        return 0f64;
    }

    let mut end_states: BTreeMap<[u8; 6],f64> = BTreeMap::new();
    for roll in rolls {
        let (tmp,_) = choose_best_field(Game(state), roll, lookup, 100);
        end_states.insert(*roll, tmp);
    }

    let mut keep_2_states: BTreeMap<[u8; 6],f64> = BTreeMap::new();
    for keep in dicekeeps {
        keep_2_states.insert(*keep, gen_keep_prob(keep, &end_states));
    }

    let mut roll_2_states: BTreeMap<[u8; 6],f64> = BTreeMap::new();
    for roll in rolls {
        let (tmp,_) = gen_roll_prob(roll,&[0,0,0,0,0,0], &keep_2_states, 100);
        roll_2_states.insert(*roll, tmp);
    }

    let mut keep_1_states: BTreeMap<[u8; 6],f64> = BTreeMap::new();
    for keep in dicekeeps {
        keep_1_states.insert(*keep, gen_keep_prob(keep, &roll_2_states));
    }

    let mut roll_1_states: BTreeMap<[u8; 6],f64> = BTreeMap::new();
    for roll in rolls {
        let (tmp,_) = gen_roll_prob(roll,&[0,0,0,0,0,0], &keep_1_states, 100);
        roll_1_states.insert(*roll, tmp);
    }

    let mut sum = 0.0;
    let mut cnt = 0.0;
    for roll in rolls {
        sum += *roll_1_states.get(roll).expect("asd");
        cnt += 1.0;
    }
    (sum / cnt)
}

pub fn gen_keep_prob(lroll: &[u8; 6], end_states: &BTreeMap<[u8; 6], f64>) -> f64 {
    let mut sum = 0f64;
    let mut cnt = 0f64;
    for ones in lroll[0]..6 {
        if ones+lroll[1]+lroll[2]+lroll[3]+lroll[4]+lroll[5] == 5 {
            sum += *(end_states.get(&[ones,lroll[1],lroll[2],lroll[3],lroll[4],lroll[5]]).expect("asdfy"));
            cnt += 1f64;
            break;
        }
        else {
            for twos in lroll[1]..6 {
                if ones+twos+lroll[2]+lroll[3]+lroll[4]+lroll[5] == 5 {
                    sum += *(end_states.get(&[ones,twos,lroll[2],lroll[3],lroll[4],lroll[5]]).expect("asdfy"));
                    cnt += 1f64;
                    break;
                }
                else {
                    for threes in lroll[2]..6 {
                        if ones+twos+threes+lroll[3]+lroll[4]+lroll[5] == 5 {
                            sum += *(end_states.get(&[ones,twos,threes,lroll[3],lroll[4],lroll[5]]).expect("asdfy"));
                            cnt += 1f64;
                            break;
                        }
                        else {
                            for fours in lroll[3]..6 {
                                if ones+twos+threes+fours+lroll[4]+lroll[5] == 5 {
                                    sum += *(end_states.get(&[ones,twos,threes,fours,lroll[4],lroll[5]]).expect("asdfy"));
                                    cnt += 1f64;
                                    break;
                                }
                                else {
                                    for fives in lroll[4]..6 {
                                        if ones+twos+threes+fours+fives+lroll[5] == 5 {
                                            sum += *(end_states.get(&[ones,twos,threes,fours,fives,lroll[5]]).expect("asdfy"));
                                            cnt += 1f64;
                                            break;
                                        }
                                        else {
                                            for sixes in lroll[5]..6 {
                                                if ones+twos+threes+fours+fives+sixes == 5 {
                                                    sum += *(end_states.get(&[ones,twos,threes,fours,fives,sixes]).expect("asdfy"));
                                                    cnt += 1f64;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    sum / cnt
}

pub fn choose_best_field(state: Game, roll: &[u8; 6], lookup: &LookupTable, skill : u32) -> (f64, u8) {
    let success_range = make_failure_range();
    let mut rng = rand::thread_rng();
    let &LookupTable(ref lookup) = lookup;
    let mut max = 0.0;
    let mut chosen_field = 0;
    for i in 0..13 {
        if state.is_free(i) {
            let Game(new_state) = state.next_turn(roll, i);
            let tmp = lookup[new_state as usize] + rules::score(roll, i) as f64;
            if tmp > max {
                if get_rand_success(&mut rng, &success_range, skill) {
                    max = tmp;
                    chosen_field = i;
                }
            }
        }
    }
    (max,chosen_field)
}

pub fn gen_roll_prob(lroll: &[u8; 6], prevroll: &[u8; 6], keep_states: &BTreeMap<[u8; 6], f64>, skill : u32) -> (f64, [u8; 6]) {
    let success_range = make_failure_range();
    let mut rng = rand::thread_rng();
    let mut max = 0.0;
    let mut maxroll = [0,0,0,0,0,0];
    for ones in 0..lroll[0]+1 {
        for twos in 0..lroll[1]+1 {
            for threes in 0..lroll[2]+1 {
                for fours in 0..lroll[3]+1 {
                    for fives in 0..lroll[4]+1 {
                        for sixes in 0..lroll[5]+1 {
                            if get_rand_success(&mut rng, &success_range, skill) {
                                let tmp2 = [ones+prevroll[0],twos+prevroll[1],threes+prevroll[2],fours+prevroll[3],fives+prevroll[4],sixes+prevroll[5]];
                                let tmp = *keep_states.get(&tmp2).expect("asdf");
                                if tmp > max { max = tmp; maxroll = tmp2; }
                            }
                        }
                    }
                }
            }
        }
    }
    (max, maxroll)
}
