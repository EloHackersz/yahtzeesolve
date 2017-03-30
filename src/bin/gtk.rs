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

extern crate gtk;
extern crate glib;
extern crate yahtzeesolve;

use std::cell::RefCell;
use gtk::traits::*;
use gtk::signal::Inhibit;
use std::sync::mpsc;
use std::thread;
use yahtzeesolve::LookupTable;
use yahtzeesolve::game::generators;
use yahtzeesolve::game::rules;
use yahtzeesolve::game::Game;

thread_local!(
    static GLOBAL: RefCell<Option<(gtk::ProgressBar, f64)>> = RefCell::new(None)
);

thread_local!(
    static GLOBAL2: RefCell<Option<(gtk::Entry, gtk::Label, gtk::ListStore)>> = RefCell::new(None)
);

thread_local!(
    static STATE: RefCell<Option<(Game, u8, [u8; 6])>> = RefCell::new(None)
);

fn main() {
    gtk::init().unwrap_or_else(|_| panic!("Failed to initialize GTK."));

    let glade_src = include_str!("gui.glade");
    let builder = gtk::widgets::Builder::new_from_string(glade_src).unwrap();

    unsafe {
        let window: gtk::Window = builder.get_object("applicationwindow1").unwrap();
        let cancel_button: gtk::Button = builder.get_object("button2").unwrap();
        let calc_button: gtk::Button = builder.get_object("button1").unwrap();
        cancel_button.connect_clicked(|_| {
            println!("asd");
            gtk::main_quit();
        });

        window.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(true)
        });
        window.show_all();

        STATE.with(move |state| {
            *state.borrow_mut() = Some((Game::new(), 0, [0,0,0,0,0,0]))
        });

        let entry1: gtk::Entry = builder.get_object("entry1").unwrap();
        let label3: gtk::Label  = builder.get_object("label3").unwrap();
        let treeview1: gtk::TreeView  = builder.get_object("treeview1").unwrap();
        let liststore = gtk::ListStore::new(&[glib::Type::String, glib::Type::U32]).unwrap();
        let listmodel = liststore.get_model().unwrap();
        treeview1.set_model(&listmodel);

        for st in vec!["Aces", "Twos", "Threes", "Fours", "Fives", "Sixes", "3 of a kind", "4 of a kind", "Full house", "Sm. straight", "Lg. straight", "YAHTZEE", "Chance"] {
            let iter = liststore.append();
            liststore.set_string(&iter, 0, st);
        }

        GLOBAL2.with(|global| {
            *global.borrow_mut() = Some((entry1, label3, liststore))
        });

        let lookup = LookupTable::from_file("probs.dat").unwrap_or_else(|_| {
            let dialog: gtk::Dialog = builder.get_object("dialog1").unwrap();
            let progressbar: gtk::ProgressBar = builder.get_object("progressbar1").unwrap();
            GLOBAL.with(move |global| {
                *global.borrow_mut() = Some((progressbar, 0.0))
            });

            let t = thread::spawn(move || {
                let (tx, rx) = mpsc::channel();
                let t2 = LookupTable::generate(tx);
                for _ in 0..100 {
                    rx.recv().unwrap();
                    glib::idle_add(|| {
                        GLOBAL.with(|global| {
                            let mut asd = global.borrow_mut();
                            match *asd {
                                Some((ref pb, ref mut progr)) => {
                                    *progr += 0.01;
                                    pb.set_fraction(*progr);
                                },
                                None => {}
                            }
                        });
                        glib::Continue(false)
                    });
                }
                t2.join().unwrap()
            });

            gtk::Dialog::run(&dialog);
            let lookup2 = t.join().unwrap();
            dialog.destroy();
            lookup2.write_to_file("probs.dat").unwrap();
            lookup2
        });
        let rollvec = generators::generate_dice_roll_possibilities();
        let dicekeeps = generators::generate_dice_keep_possibilities();

        calc_button.connect_clicked(move |_| {
            STATE.with(|state| {
                let mut asd = state.borrow_mut();
                match *asd {
                    None => {},
                    Some((ref mut state, ref mut cnt, ref mut already_rolled)) => {
                        GLOBAL2.with(|global| {
                            if let Some((ref entry, ref label, ref liststore)) = *global.borrow() {
                                calc_round(state, cnt, already_rolled, &rollvec, &dicekeeps, &lookup, entry, label, liststore);
                            }
                        })
                    }
                }
            });
        });
    }

    gtk::main();
}

fn calc_round(game: &mut Game, cnt: &mut u8, already_rolled: &mut [u8; 6], rollvec: &Vec<[u8; 6]>, dicekeeps: &Vec<[u8; 6]>, lookup: &LookupTable, entry: &gtk::Entry, label: &gtk::Label, liststore: &gtk::ListStore) {
    // todo: save this between states
    let (keep_1_states, keep_2_states) = yahtzeesolve::precalc_current_round(*game, lookup, rollvec, dicekeeps, 100);

    let newroll = entry.get_text().unwrap();
    let input: u32 = newroll.parse().unwrap();
    let inp = key_conv(input);

    match *cnt {
        0 => {
            let (_,kroll) = generators::gen_roll_prob(&inp, already_rolled, &keep_1_states, 100);
            let mut keepstr = format!("Keep ");
            let mut beginning = true;
            for i in 0..6 {
                already_rolled[i] = kroll[i];
                if kroll[i] != 0 {
                    if beginning {
                        beginning = false;
                    }
                    else {
                        keepstr = keepstr + ", ";
                    }
                    keepstr = keepstr + &format!("{} x {}", kroll[i], i + 1);
                }
            }
            if already_rolled.iter().fold(0, |a, &b| a + b) == 5 {
                let (_,choseni) = generators::choose_best_field(*game, already_rolled, lookup, 100);
                let scr = rules::score(already_rolled, choseni);
                *game = game.next_turn(already_rolled, choseni);
                *cnt = 0;
                let path = gtk::TreePath::new_from_indicesv(&mut [choseni as i32]).unwrap();
                let treemodel: gtk::TreeModel = liststore.get_model().unwrap();
                let iter = treemodel.get_iter(&path).unwrap();
                unsafe {
                    let mut val = glib::Value::new();
                    val.init(glib::Type::U32);
                    val.set_uint(scr as u32);
                    liststore.set_value(&iter, 1, &val);
                }
                *already_rolled = [0,0,0,0,0,0];
                let fieldlabels = vec!["Aces", "Twos", "Threes", "Fours", "Fives", "Sixes", "3 of a kind", "4 of a kind", "Full house", "Sm. straight", "Lg. straight", "YAHTZEE", "Chance"];
                let textlabel = format!("Put {} in the {} field. Roll.", scr, fieldlabels[choseni as usize]);
                label.set_text(&textlabel);
                entry.set_text("");
                return;
            }

            if beginning {
                keepstr = "Roll again.".to_string();
            }
            else {
                keepstr = keepstr + ". Roll again.";
            }
            label.set_text(&keepstr);
            *cnt += 1;
        },
        1 => {
            let (_,kroll) = generators::gen_roll_prob(&inp, already_rolled, &keep_2_states, 100);
            let mut keepstr = format!("Keep ");
            let mut beginning = true;
            for i in 0..6 {
                if kroll[i] != 0 && kroll[i] != already_rolled[i] {
                    if beginning {
                        beginning = false;
                    }
                    else {
                        keepstr = keepstr + ", ";
                    }
                    keepstr = keepstr + &format!("{} x {}", kroll[i] - already_rolled[i], i + 1);
                }
                already_rolled[i] = kroll[i];
            }
            if already_rolled.iter().fold(0, |a, &b| a + b) == 5 {
                let (_,choseni) = generators::choose_best_field(*game, already_rolled, lookup, 100);
                let scr = rules::score(already_rolled, choseni);
                *game = game.next_turn(already_rolled, choseni);
                *cnt = 0;
                let path = gtk::TreePath::new_from_indicesv(&mut [choseni as i32]).unwrap();
                let treemodel: gtk::TreeModel = liststore.get_model().unwrap();
                let iter = treemodel.get_iter(&path).unwrap();
                unsafe {
                    let mut val = glib::Value::new();
                    val.init(glib::Type::U32);
                    val.set_uint(scr as u32);
                    liststore.set_value(&iter, 1, &val);
                }
                *already_rolled = [0,0,0,0,0,0];
                let fieldlabels = vec!["Aces", "Twos", "Threes", "Fours", "Fives", "Sixes", "3 of a kind", "4 of a kind", "Full house", "Sm. straight", "Lg. straight", "YAHTZEE", "Chance"];
                let textlabel = format!("Put {} in the {} field. Roll.", scr, fieldlabels[choseni as usize]);
                label.set_text(&textlabel);
            }

            if beginning {
                keepstr = "Roll again.".to_string();
            }
            else {
                keepstr = keepstr + ". Roll again.";
            }
            label.set_text(&keepstr);

            *cnt += 1;
        },
        2 => {
            for i in 0..6 {
                already_rolled[i] += inp[i];
            }
            let (_,choseni) = generators::choose_best_field(*game, already_rolled, lookup, 100);
            let scr = rules::score(already_rolled, choseni);
            *game = game.next_turn(already_rolled, choseni);
            *cnt = 0;
            let path = gtk::TreePath::new_from_indicesv(&mut [choseni as i32]).unwrap();
            let treemodel: gtk::TreeModel = liststore.get_model().unwrap();
            let iter = treemodel.get_iter(&path).unwrap();
            unsafe {
                let mut val = glib::Value::new();
                val.init(glib::Type::U32);
                val.set_uint(scr as u32);
                liststore.set_value(&iter, 1, &val);
            }
            *already_rolled = [0,0,0,0,0,0];
            let fieldlabels = vec!["Aces", "Twos", "Threes", "Fours", "Fives", "Sixes", "3 of a kind", "4 of a kind", "Full house", "Sm. straight", "Lg. straight", "YAHTZEE", "Chance"];
            let textlabel = format!("Put {} in the {} field. Roll.", scr, fieldlabels[choseni as usize]);
            label.set_text(&textlabel);
        },
        _ => {}
    }
    entry.set_text("");
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
