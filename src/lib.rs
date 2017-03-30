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
extern crate byteorder;

use game::generators;
use game::Game;
pub use game::lookuptable::LookupTable;
use std::collections::BTreeMap;

pub mod game;

pub fn precalc_current_round(game: Game, lookup: &LookupTable, rollvec: &Vec<[u8; 6]>, dicekeeps: &Vec<[u8; 6]>, skill : u32) -> (BTreeMap<[u8; 6],f64>, BTreeMap<[u8; 6],f64>) {
    let mut end_states: BTreeMap<[u8; 6],f64> = BTreeMap::new();
    for roll in rollvec {
        let (tmp,_) = generators::choose_best_field(game, roll, lookup, skill);
        end_states.insert(*roll, tmp);
    }

    let mut keep_2_states: BTreeMap<[u8; 6],f64> = BTreeMap::new();
    for keep in dicekeeps {
        keep_2_states.insert(*keep, generators::gen_keep_prob(keep, &end_states));
    }

    let mut roll_2_states: BTreeMap<[u8; 6],f64> = BTreeMap::new();
    for roll in rollvec {
        let (tmp,_) = generators::gen_roll_prob(roll,&[0,0,0,0,0,0], &keep_2_states, skill);
        roll_2_states.insert(*roll, tmp);
    }

    let mut keep_1_states: BTreeMap<[u8; 6],f64> = BTreeMap::new();
    for keep in dicekeeps {
        keep_1_states.insert(*keep, generators::gen_keep_prob(keep, &roll_2_states));
    }

    (keep_1_states, keep_2_states)
}
