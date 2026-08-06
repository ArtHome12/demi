/* ===============================================================================
Simulation of the evolution of the animal world.
Speed calculation (FPS, model time).
07 Mar 2021.
----------------------------------------------------------------------------
Licensed under the terms of the GPL version 3.
http://www.gnu.org/licenses/gpl-3.0.html
Copyright (c) 2013-2022 by Artem Khomenko _mag12@yahoo.com.
=============================================================================== */

// Struct for counting FPS, screen refresh rate
#[derive(Default)]
pub struct FPS {
   pub rate: usize, // Last computed value
   ticks: usize,  // Number of ticks since the beginning of a second
}

impl FPS {
   // Update counter
   pub fn make_tick(&mut self) {
      self.ticks += 1;
   }

   // Start new second
   pub fn clock_chime(&mut self) {
      self.rate = self.ticks;
      self.ticks = 0;
   }
}

// Struct for counting TPS, model time rate, ticks per second
#[derive(Default)]
pub struct TPS {
   pub rate: usize, // Last computed value
   ticks: usize,  // Number of ticks at last clock_chime()
}

impl TPS {
   // Start new second with new ticks amount
   pub fn clock_chime(&mut self, new_tick_number: usize) {
      self.rate = new_tick_number - self.ticks;
      self.ticks = new_tick_number;
   }
}