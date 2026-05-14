/* ===============================================================================
Simulation of the evolution of the animal world.
Genes.
11 Mar 2026.
----------------------------------------------------------------------------
Licensed under the terms of the GPL version 3.
http://www.gnu.org/licenses/gpl-3.0.html
Copyright (c) 2013-2022 by Artem Khomenko _mag12@yahoo.com.
=============================================================================== */

use bitvec::prelude::*;
use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, PartialEq)]
pub enum State {
   On,
   Off,
   Any,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Line {
   // 0 = dont care, 1 = defined (on or off)
   pub defined: BitVec,
   // 0 = off, 1 = on
   pub state: BitVec,
}

impl Line {
   pub fn new(size: usize) -> Self {
      Self {
         defined: bitvec![0; size],
         state: bitvec![0; size],
      }
   }


   pub fn set(&mut self, index: usize, state: State) {
      match state {
         State::On => {
            // defined as enabled (required)
            self.defined.set(index, true);
            self.state.set(index, true);
         },
         State::Off => {
            // defined as disabled
            self.defined.set(index, true);
            self.state.set(index, false);
         },
         State::Any => {
            // not defined, don't care
            self.defined.set(index, false);
         },
      }
   }


   pub fn get(&self, index: usize) -> State {
      if self.defined[index] {
         if self.state[index] {
            State::On
         } else {
            State::Off
         }
      } else {
         State::Any
      }
   }


   pub fn matches(&self, other: &Line) -> bool {
      // Check if all defined bits in self match the corresponding bits in other
      for (i, defined) in self.defined.iter().enumerate() {
         if *defined {
            if self.state[i] != other.state[i] {
               return false;
            }
         }
      }
      true
   }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FSM {
   pub index: usize, // index of the active line
   pub lines: Vec<Line>,
}