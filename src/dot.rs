/* ===============================================================================
Simulation of the evolution of the animal world.
Piece of territory.
17 Feb 2021.
----------------------------------------------------------------------------
Licensed under the terms of the GPL version 3.
http://www.gnu.org/licenses/gpl-3.0.html
Copyright (c) 2013-2022 by Artem Khomenko _mag12@yahoo.com.
=============================================================================== */

use std::ptr;
use iced::Color;
use serde::{Serialize, Deserialize};

use crate::geom::Size;

// Representation for display
#[derive(Debug, Clone, Copy)]
pub struct Dot {
   // Location
   pub x: usize,
   pub y: usize,

   // Color for display
   pub color: Color,
}

// Storage amount by points of one element
#[derive(Serialize, Deserialize)]
pub struct ElementsSheet {
   pub matrix: Vec<usize>,

   // How much the fraction moves during diffusion, from 0 to 1 for elements and <0 for energy
   pub volatility: f32,
}

impl ElementsSheet {
   #[must_use]
   pub fn new(size: Size, initial_amount: usize, volatility: f32) -> Self {

      // Amount of points
      let prod = size.x * size.y;

      Self {
         matrix: vec![initial_amount; prod],
         volatility,
      }
   }

   pub fn set(&mut self, i: usize, amount: usize) {
      self.matrix[i] = amount;
   }
}

#[derive(Serialize, Deserialize)]
pub struct ElementsSheets(Vec<ElementsSheet>);

impl ElementsSheets {
   #[must_use]
   pub fn get(&self) -> &Vec<ElementsSheet> {
      &self.0
   }

   pub fn get_mut(&mut self) -> &mut Vec<ElementsSheet> {
      &mut self.0
   }
}

impl std::iter::FromIterator<ElementsSheet> for ElementsSheets {
   fn from_iter<I: IntoIterator<Item = ElementsSheet>>(iter: I) -> Self {
      Self(iter.into_iter().collect())
   }
}

#[derive(Debug)]
// Fast unsafe access to elements for mirroring and rayon
pub struct PtrElements(Vec<usize>);

impl PtrElements {
   #[must_use]
   pub fn new(sheets: &ElementsSheets) -> Self {
      // Store raw pointers to elements
      let ptr = sheets.get().iter()
      .map(|sheet| ptr::addr_of!(sheet.matrix[0]) as usize)
      .collect();

      Self(ptr)
   }

   /// Read the amount of `element_index` element at `serial` point.
   ///
   /// # Safety
   /// `element_index` must reference a valid sheet and `serial` must be within
   /// that sheet's bounds; otherwise the raw pointer arithmetic reads out of
   /// bounds. The backing `ElementsSheets` must stay alive and unmoved while the
   /// pointer view is in use.
   #[must_use]
   pub unsafe fn get(&self, element_index: usize, serial: usize) -> usize {
      unsafe { (self.0[element_index] as *const usize).add(serial).read() }
   }

   /// Increase by `delta` the amount of `element_index` element at `serial` point.
   ///
   /// # Safety
   /// Same invariants as [`Self::get`]: valid `element_index`/`serial` and a live,
   /// stable `ElementsSheets` backing store.
   pub unsafe fn inc_amount(&self, element_index: usize, serial: usize, delta: usize) {
      unsafe {
         let dest = (self.0[element_index] as *mut usize).add(serial);
         let new_val = dest.read().saturating_add(delta);
         std::ptr::write(dest, new_val);
      }
   }

   /// Decrease by `delta` the amount of `element_index` element at `serial` point.
   ///
   /// # Safety
   /// Same invariants as [`Self::get`]: valid `element_index`/`serial` and a live,
   /// stable `ElementsSheets` backing store.
   pub unsafe fn dec_amount(&self, element_index: usize, serial: usize, delta: usize) {
      unsafe {
         let dest = (self.0[element_index] as *mut usize).add(serial);
         let new_val = dest.read().saturating_sub(delta);
         std::ptr::write(dest, new_val);
      }
   }
}