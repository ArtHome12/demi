/* ===============================================================================
Simulation of the evolution of the animal world.
Mechanism of evolution for the world.
03 Mar 2021.
----------------------------------------------------------------------------
Licensed under the terms of the GPL version 3.
http://www.gnu.org/licenses/gpl-3.0.html
Copyright (c) 2013-2022 by Artem Khomenko _mag12@yahoo.com.
=============================================================================== */

use crate::geom::*;
use crate::organism::Organism;
use crate::genes::*;

#[derive(Debug, Clone)]
pub struct Environment {
   // World size
   pub world_size: Size,

	// The maximum illumination radius around the location of the Sun is 90% of the height of the world
	pub light_radius: usize,

   // The course of the Sun vertically, due to the inclination of the earth axis, approximately sin (23.5) = 0.4 or 10% on each side of the equator
	pub tropic_height: f32,

   // How many bits to move (diffusion) elements in one tick
   pub num_points_to_diffuse: usize,

   // Range for serial number of all bits
   pub bits_count: usize,

   // First organism
   pub luca: Organism,

   // Color of heterotrophs
   pub heterotroph_color: iced::Color,
}

impl Environment {
   // Number of ticks per day and days per year
   const TICKS_PER_DAY: f32 = 60.0*24.0;
   const DAYS_PER_YEAR: f32 = 365.0;
   const TICKS_PER_YEAR: f32 = Self::TICKS_PER_DAY * Self::DAYS_PER_YEAR;
   const HALF_YEAR: f32 = Self::DAYS_PER_YEAR / 2.0 + 0.5;

   pub fn new(world_size: Size, resolution: f32, luca_reaction: usize, number_of_reactions: usize, heterotroph_color: iced::Color) -> Self {
      let world_height = world_size.y;

      // How many bits to move (diffusion) elements in one tick
      let bits_count = world_size.x * world_height;
      let num_points_to_diffuse = (bits_count as f32 * resolution) as usize;

      // Create first organism
      let vitality = 3 * world_size.x;   // hold out for 3 days
      let level = 3 * vitality;  // grow 3 times to start breeding
      let luca = Organism::new(vitality, 0,
         Digestion::new(luca_reaction, number_of_reactions, NutritionMode::Autotroph),
         Reproduction::new(level),
      );

      Self {
         world_size,
         light_radius: (0.8 * world_height as f32 / 2.0) as usize,
         tropic_height: world_height as f32 / 5.0,
         num_points_to_diffuse,
         bits_count,
         luca,
         heterotroph_color,
      }
   }

   // Returns the position of the sun at the specified time
   pub fn sun_position(&self, tick: usize) -> Coord {
      // Times of Day
      let day_tick = tick as f32 % Self::TICKS_PER_DAY;

      let Size{x: w, y: h} = self.world_size;

      // The sun moves from east to west in proportion to the elapsed fraction of a day
      let x = w as f32 * (1.0 - day_tick / (Self::TICKS_PER_DAY - 1.0) + 0.5);

      // When the first half of the year goes, it is necessary to take a share from the equator, and when the second is to add
      let day = tick as f32 % Self::TICKS_PER_YEAR;
      let y = if day < Self::HALF_YEAR {
		   (h as f32 - self.tropic_height) / 2.0 + day * self.tropic_height / Self::HALF_YEAR
      } else {
		   (h as f32 + self.tropic_height) / 2.0 - (day - Self::HALF_YEAR) * self.tropic_height / Self::HALF_YEAR
      };

      Coord::new(x as usize, y as usize)
   }

   pub fn date(tick: usize) -> (usize, usize) {
      let days = tick as f32 / Self::TICKS_PER_DAY;
      let years = (days / Self::DAYS_PER_YEAR) as usize;
      let days = (days % Self::DAYS_PER_YEAR) as usize;
      (years, days)
   }

   // Return serial number of bit at direction without checking bounds
   pub fn distance(&self, direction: Direction) -> isize {
      let width = self.world_size.x as isize;

      match direction {
         Direction::North => -width,
         Direction::Northeast => 1 - width,
         Direction::East => 1,
         Direction::Southeast => 1 + width,
         Direction::South => width,
         Direction::Southwest => -1 + width,
         Direction::West => -1,
         Direction::Northwest => -1 - width,
         Direction::None => 0,
      }
   }
}