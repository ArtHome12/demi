/* ===============================================================================
Simulation of the evolution of the animal world.
Genes.
22 Apr 2021.
----------------------------------------------------------------------------
Licensed under the terms of the GPL version 3.
http://www.gnu.org/licenses/gpl-3.0.html
Copyright (c) 2013-2022 by Artem Khomenko _mag12@yahoo.com.
=============================================================================== */

use rand::prelude::*;
use rand::distr::Uniform;
use serde::{Serialize, Deserialize};

pub trait Gene {
   fn mutate(&self, rng: &mut ThreadRng) -> Self;
}


#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum NutritionMode {
   Autotroph,  // autotrophs (plants) - use light energy and elements to produce food
   Heterotroph,   // heterotrophs (animals) - eats autotrophs or other heterotrophs
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "RawDigestion")]
pub struct Digestion {
   pub reaction: usize, // reaction index
   number_of_reactions: usize,
   #[serde(skip)]
   variability: Uniform<usize>, // number of reactions * 3
   pub mode: NutritionMode,
}


#[derive(Deserialize)]
pub struct RawDigestion {
   pub reaction: usize, // reaction index
   number_of_reactions: usize,
   pub mode: NutritionMode,
}


impl From<RawDigestion> for Digestion {
   fn from(raw: RawDigestion) -> Self {
      Self::new(raw.reaction, raw.number_of_reactions, raw.mode)
   }
}


impl Digestion {
   pub fn new(reaction: usize, number_of_reactions: usize, mode: NutritionMode) -> Self {
      Self {
         reaction,
         number_of_reactions,
         variability: Uniform::new(0, number_of_reactions * 3).unwrap(),
         mode,
      }
   }
}

impl Gene for Digestion {
   fn mutate(&self, rng: &mut ThreadRng) -> Self {

      // Prepare a clone
      let mut res = self.clone();

      // Determine whether there will be a mutation
      let dice = self.variability.sample(rng);

      // Change the reaction or the mode with equal probability
      if dice == self.reaction {
         res.mode = match self.mode {
            NutritionMode::Autotroph => NutritionMode::Heterotroph,
            NutritionMode::Heterotroph => NutritionMode::Autotroph,
         }
      } else if dice < self.number_of_reactions {
         res.reaction = dice;
      }

      res
   }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reproduction {
   pub level: usize, // the level of strength (vitality) required to divide
   #[serde(skip, default="Reproduction::default_variability")]
   variability: Uniform<usize>, // 0..=2
}


impl Reproduction {
   pub fn new(level: usize) -> Self {
      Self {
         level,
         variability: Uniform::try_from(0..=2).unwrap(),
      }
   }

   fn default_variability() -> Uniform<usize> {
      Uniform::try_from(0..=2).unwrap()
   }
}


impl Gene for Reproduction {
   fn mutate(&self, rng: &mut ThreadRng) -> Self {
      let level = match self.variability.sample(rng) {
         0 => self.level.saturating_sub(1),
         1 => self.level.saturating_add(1),
         _ => self.level,
      };

      Self {
         level,
         variability: self.variability.clone(),
      }
   }
}
