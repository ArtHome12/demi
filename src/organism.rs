/* ===============================================================================
Simulation of the evolution of the animal world.
Organism.
22 Apr 2021.
----------------------------------------------------------------------------
Licensed under the terms of the GPL version 3.
http://www.gnu.org/licenses/gpl-3.0.html
Copyright (c) 2013-2022 by Artem Khomenko _mag12@yahoo.com.
=============================================================================== */

use std::{cmp::max, fmt, ptr, marker::PhantomData};
use rand::prelude::ThreadRng;
use serde::{Serialize, Deserialize};

use crate::genes::{Digestion, Reproduction, Gene, NutritionMode};
use crate::reactions::Reactions;
use crate::dot::PtrElements;
use crate::state::FSM;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organism {
   pub vitality: usize,
   pub birthday: usize,
   gene_digestion: Digestion,
   metabolism: usize,
   gene_reproduction: Reproduction,
   fcm: FSM,
}

impl Organism {
   #[must_use]
   pub fn new(vitality: usize, birthday: usize, gene_digestion: Digestion, gene_reproduction: Reproduction, fcm: FSM) -> Self {
      Self {
         vitality,
         birthday,
         metabolism: 0,
         gene_digestion,
         gene_reproduction,
         fcm,
      }
   }


   #[must_use]
   pub fn alive(&self, ) -> bool {
      self.vitality > 0
   }


   pub fn digestion_autotroph(&mut self, ptr: &PtrElements, serial: usize, reactions: &Reactions) {
      // Check the availability of resources for digestion
      let r = self.gene_digestion.reaction;
      let r = reactions.get(r);
      let avail = r.left.iter().all(|reagent| {
            reagent.amount <= unsafe { ptr.get(reagent.index, serial) }
         });
      if !avail {
         return
      }

      // Process the reaction

      // Subtract source elements
      r.left.iter()
      .for_each(|reagent| {
         unsafe { ptr.dec_amount(reagent.index, serial, reagent.amount) };
      });

      // Add the resulting elements
      r.right.iter()
      .for_each(|reagent| {
         unsafe { ptr.inc_amount(reagent.index, serial, reagent.amount) };
      });

      // Increase vitality
      self.vitality = self.vitality.saturating_add(r.vitality);
   }


   pub fn digestion_heterotroph(&mut self, food: &Organism, reactions: &Reactions) {
      // For first part of portion
      let r = self.gene_digestion.reaction;
      let r = reactions.get(r);

      let portion = r.vitality + food.gene_reproduction.level / 2;
      
      // Increase vitality
      self.vitality = self.vitality.saturating_add(portion);
   }


   pub fn reproduction(&mut self, rng: &mut ThreadRng) -> Option<Organism> {
      // Check avaliability
      let level = self.gene_reproduction.level;
      if self.vitality > level {
         // Reducing own reserves of vitality
         self.vitality -= level;

         // Bring into the world a new organism
         let res = Self {
            vitality: level / 2,
            birthday: 0,
            metabolism: 0,
            gene_digestion: self.gene_digestion.mutate(rng),
            gene_reproduction: self.gene_reproduction.mutate(rng),
            fcm: self.fcm.clone(),
         };
         Some(res)

      } else {
         None
      }
   }


   #[must_use]
   pub fn reaction_index(&self) -> usize {
      self.gene_digestion.reaction
   }

   
   #[must_use]
   pub fn nutrition_mode(&self) -> NutritionMode {
      self.gene_digestion.mode
   }
}


impl fmt::Display for Organism {
   fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       write!(f, "{}˄ {}Δ", self.vitality, self.gene_reproduction.level)
   }
}


// Organisms at one point
#[derive(Serialize, Deserialize)]
pub struct AnimalsStack {
   stack: Vec<Option<Organism>>,
}

impl<'s> AnimalsStack {

   #[must_use]
   pub fn new(max_animal_stack: usize) -> Self {
      Self{
         stack: vec![None; max_animal_stack],
      }
   }

   // Returns iterator over alive organism in stack, if such exists
   pub fn get_mut_alive(&'s mut self) -> impl Iterator<Item = &'s mut Organism> {
      self.stack
      .iter_mut()
      .filter_map(|item| item.as_mut().and_then(|o| o.alive().then_some(o)))
   }


   // Return index of first empty or occuped item
   #[must_use]
   pub fn get_slot_index(&self, occuped: bool) -> Option<usize> {
      self.stack
      .iter()
      .position(|opt| opt.is_some() == occuped)
   }


   pub fn reproduction(&mut self, now: usize) {
      // Checking for free space at the point
      let free_slot = self.get_slot_index(false);
      if let Some(free_slot) = free_slot {

         // For optimization
         let mut rng = rand::rng();

         // Looking for among all living organisms at a point capable of reproducing. The first gets priority
         let opt_new_born = self
         .get_mut_alive()
         .find_map(|animal| animal.reproduction(&mut rng));

         // Ask about birth
         if let Some(mut new_born) = opt_new_born {
            // Place new organism
            new_born.birthday = now;
            self.stack[free_slot] = Some(new_born);
         }
      }
   }


   pub fn end_of_turn(&mut self, now: usize) {
      // Each alive organism at the point
      self.get_mut_alive()
      .for_each(|animal| {
         // Decrease vitality on methabolism
         animal.vitality = animal.vitality.saturating_sub(max(1, animal.metabolism));

         // In case of death keep the age
         if animal.vitality == 0 {
            animal.birthday = now - animal.birthday + 1;
         }
      });
   }


   pub fn digestion(&mut self, ptr_elements: &PtrElements, ptr_animals: &PtrAnimals, serial: usize, reactions: &Reactions) {
      self.get_mut_alive()
      .for_each(|animal| {
         match animal.gene_digestion.mode {
            NutritionMode::Autotroph => animal.digestion_autotroph(ptr_elements, serial, reactions),
            NutritionMode::Heterotroph => {
               let food = ptr_animals.get_stack(serial).get_dead();
               if let Some(food) = food {
                  animal.digestion_heterotroph(&food, reactions);
               }
            }
         }
      });
   }
}


#[derive(Serialize, Deserialize)]
// Keeps live and death organisms in the grid
pub struct AnimalsSheet {
   pub sheet: Vec<AnimalsStack>
}


impl AnimalsSheet {
   #[must_use]
   pub fn new(max_serial: usize, max_animal_stack: usize) -> Self {
      let mut sheet = Vec::with_capacity(max_serial);

      // Fill points with stacks for futures animals
      (0..max_serial)
      .for_each(|_| sheet.push(AnimalsStack::new(max_animal_stack)));

      Self {
         sheet,
      }
   }


   // Return stack of organisms at point
   #[must_use]
   pub fn get(&self, index: usize) -> &AnimalsStack {
      &self.sheet[index]
   }


   pub fn get_mut(&mut self, index: usize) -> &mut AnimalsStack {
      &mut self.sheet[index]
   }


   // Try to transfer organism between points
   pub fn transfer(&mut self, origin: usize, dest: usize) {

      // If there is something to transfer and free space at destinaton
      if let Some(source_index) = self.get(origin).get_slot_index(true)
         && let Some(dest_index) = self.get(dest).get_slot_index(false) {

            // Extract organism from previous place
            let origin_stack = self.get_mut(origin);
            let origin_place = &mut origin_stack.stack[source_index];
            let organism = origin_place.take();

            // And put it to the new place
            let dest_stack = self.get_mut(dest);
            dest_stack.stack[dest_index] = organism;
         }
   }


   pub fn implantation(&mut self, luca: &Organism, now: usize) {
      let stack = self.get_mut(0);
      let first_alive = stack.get_mut_alive().next();
      if first_alive.is_none() {

         // Check free place
         if let Some(free_slot) = stack.get_slot_index(false) {
            let mut luca = luca.to_owned();
            luca.birthday = now;
            stack.stack[free_slot] = Some(luca);
         }
      }
   }
}


#[derive(Debug)]
// Pointers to fast unsafe access to organisms
pub struct PtrAnimals {
   ptr: Vec<usize>,
   stack_size: usize,
}


impl PtrAnimals {
   #[must_use]
   pub fn new(animals: &AnimalsSheet) -> Self {
      // Store raw pointers to elements
      let ptr = animals.sheet.iter()
      .map(|stack| ptr::addr_of!(stack.stack[0]) as usize)
      .collect();

      Self {
         ptr,
         stack_size: animals.sheet[0].stack.len(),
      }
   }

   #[must_use]
   pub fn get_stack(&self, serial: usize) -> PtrAnimalsStack<'_> {
      PtrAnimalsStack {
         start: self.ptr[serial],
         curr: 0,
         len: self.stack_size,
         phantom: PhantomData,
      }
   }
}


#[derive(Clone, Debug)]
// Organisms at one point
pub struct PtrAnimalsStack<'a> {
   start: usize,  // address of first element in stack
   curr: usize,   // The ordinal number of the next element of the iterator
   len: usize,    // The number of elements in the stack
   phantom: PhantomData<&'a Option<Organism>>
}


impl<'a> PtrAnimalsStack<'a> {
   
   // Returns a mutable iterator over the stack
   pub fn iter_mut<'b>(&'b mut self) -> impl Iterator<Item = &'b mut Option<Organism>> + use<'a, 'b> {
      (0..self.len).map(move |i| unsafe {
         (self.start as *mut Option<Organism>)
            .add(i)
            .as_mut()
            .unwrap() // replace with as_mut_unchecked() once it is stable
      })
   }

   // Returns the first non-living organism, leaving None in its place
   pub fn get_dead(&'a mut self) -> Option<Organism> {
      let res = self.iter_mut()
      .find(|opt| opt.as_ref().is_some_and(|o| !o.alive()));

      match res {
         Some(opt) => opt.take(),
         None => None,
      }
   }
}

impl<'a> Iterator for PtrAnimalsStack<'a> {
   type Item = &'a Option<Organism>;

   fn next(&mut self) -> Option<Self::Item> {
      if self.curr < self.len {
         let res = unsafe{ (self.start as *const Option<Organism>)
            .add(self.curr)
            .as_ref()
            .unwrap()   // replace with as_ref_unchecked() once it is stable
         };
         self.curr += 1;

         Some(res)
      } else {
         None
      }
   }
}