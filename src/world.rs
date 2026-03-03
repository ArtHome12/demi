/* ===============================================================================
Simulation of the evolution of the animal world.
The world as a set of dots.
17 Feb 2021.
----------------------------------------------------------------------------
Licensed under the terms of the GPL version 3.
http://www.gnu.org/licenses/gpl-3.0.html
Copyright (c) 2013-2023 by Artem Khomenko _mag12@yahoo.com.
=============================================================================== */

use std::{fs::File, path::PathBuf, sync::{Arc, atomic::{AtomicU8, AtomicUsize, Ordering}, }, };
use std::time::{Duration, Instant,};
use std::io::{BufReader, BufWriter,};

use crate::{dot::ElementsSheet, evolution::Evolution, environment::*};
pub use crate::dot::{Dot, ElementsSheets, PtrElements};
use crate::geom::*;
use crate::project::{Project, Element, };
use crate::organism::*;
use crate::reactions::UIReactions;
use crate::genes::NutritionMode;

type Handle = std::thread::JoinHandle<()>;

#[derive(Debug)]
pub struct World {
   env: Environment,
   mode: Arc<AtomicU8>,
   ticks_elapsed: Arc<AtomicUsize>, // model time - a number ticks elapsed from beginning
   ptr_elements: PtrElements,       // Contents of inanimate nature calculations
   ptr_animals: PtrAnimals,         // Contents of living nature calculations
   thread_handle: Option<Handle>,

   // Section for UI
   pub vis_elem_indexes: Vec<bool>, // indexes of visible (non-filtered) elements
   pub vis_reac_indexes: Vec<bool>, // indexes of visible (non-filtered) reactions
   pub show_dead: bool,              // to show or not dead organisms
   pub show_heterotrophs: bool,       // to show or not heterotrophs
   pub ui_reactions: UIReactions,   // description of reactions
   pub ui_elements: Vec<Element>,      // description of elements
}

// Modes of operation of the calculated thread
#[derive(Copy, Clone, PartialEq)]
enum ThreadMode {
   Run = 1, // evaluate tick by tick
   Pause = 2, // command from main to pause
   Paused = 3, // signal from thread about pause
   Shutdown = 4, // command from main to break loop
   Save = 5, // command from main to save project
   Load = 6, // command from main to indicate about load project
}

impl From<u8> for ThreadMode {
   fn from(n: u8) -> Self {
      match n {
         1 => ThreadMode::Run,
         2 => ThreadMode::Pause,
         3 => ThreadMode::Paused,
         4 => ThreadMode::Shutdown,
         5 => ThreadMode::Save,
         6 => ThreadMode::Load,
         _ => panic!("world::ThreadMode({})", n),
      }
   }
}

impl World {
   pub fn new(project: Project, load_bin: bool) -> Self {
      // Initialize structures from project
      let ui_elements = project.elements;
      let reactions = project.reactions;
      let ui_reactions = project.ui_reactions;
      let size = project.size;

      let env = Environment::new(size,
         project.resolution,
         project.luca_reaction,
         reactions.len(),
         project.heterotroph_color,
      );

      // Prepare filename for binary file
      let filename = project.filename.with_extension("demi");

      // Read or create
      let (elements, animals, tick) = {
         // Helper function to create initial state to avoid duplication
         let create_initial_state = || {
            // Create sheets of elements with initial amounts
            let elements = ui_elements.iter().map(|v| {
               ElementsSheet::new(size, v.init_amount, v.volatility)
            }).collect::<ElementsSheets>();

            // Create animals
            let animals = AnimalsSheet::new(size.max_serial(), project.max_animal_stack);
            (elements, animals, 0)
         };

         if load_bin {
            match Self::load_compressed_data(&filename) {
               Ok((elements, animals, tick)) => (elements, animals, tick),
               Err(e) => {
                  // Display an error message if the file exists but the load fails.
                  if !matches!(e, FileError::NotFound) {
                     eprintln!("Could not load project from {}: {:?}. Starting with initial state.", filename.display(), e);
                  }
                  create_initial_state()
               }
            }
         } else {
            // Если load_bin false, сразу создаем начальное состояние без попытки загрузки
            create_initial_state()
         }
      };

      // Flags for thread control
      let mode = Arc::new(AtomicU8::new(ThreadMode::Paused as u8));

      // The model time
      let ticks_elapsed = Arc::new(AtomicUsize::new(tick));

      // Thread for calculate evolution

      // Evolution algorithm
      let evolution = Evolution::new(elements, animals, reactions);

      // Store raw pointers to content
      let ptr_elements = PtrElements::new(&evolution.elements);
      let ptr_animals = PtrAnimals::new(&evolution.animals);

      let thread_handle = Self::spawn(
         env.clone(),
         evolution,
         mode.clone(),
         ticks_elapsed.clone(),
         filename,
      );

      // At start all elements should be visible, collect its indexes
      let len = ui_elements.len();
      let vis_elem_indexes = vec![true; len];
      let vis_reac_indexes = vec![true; len];

      Self {
         env,
         mode,
         ticks_elapsed,
         ptr_elements,
         ptr_animals,
         thread_handle,

         vis_elem_indexes,
         vis_reac_indexes,
         show_dead: true,
         show_heterotrophs: true,
         ui_reactions,
         ui_elements,
      }
   }

   fn spawn(env: Environment, mut evolution: Evolution, mode: Arc<AtomicU8>, ticks: Arc<AtomicUsize>, filename: PathBuf) -> Option<Handle> {
      // Thread for calculate evolution
      let thread_handle = std::thread::spawn(move || {
         let sleep_time = Duration::from_millis(100);

         // For resume after save
         let mut prev_state: ThreadMode = mode.load(Ordering::Relaxed).into();

         // Running until program not closed
         loop {
            // Get task
            let task = mode.load(Ordering::Acquire).into();
            match task {
               ThreadMode::Run => {
                  prev_state = ThreadMode::Run;

                  // Increase model time
                  let tick = ticks.fetch_add(1, Ordering::Relaxed);

                  // Calculate the tick of evolution and try to mirror data
                  evolution.make_tick(&env, tick);
               }
               ThreadMode::Pause => {
                  prev_state = ThreadMode::Paused;
                  mode.store(ThreadMode::Paused as u8, Ordering::Relaxed);
               }
               ThreadMode::Paused | ThreadMode::Load => std::thread::sleep(sleep_time),
               ThreadMode::Shutdown => {
                  mode.store(ThreadMode::Paused as u8, Ordering::Relaxed);
                  break;
               }
               ThreadMode::Save => {
                  // Save project
                  {
                     let tick = ticks.fetch_add(1, Ordering::Relaxed);
                     if let Err(e) = Self::save_compressed_data(&filename, &evolution, tick) {
                        eprintln!("Error saving project to {}: {:?}", filename.display(), e);
                     }
                  }

                  // Resume previous state, before save
                  mode.store(prev_state as u8, Ordering::Relaxed);
               }
            }
         }
   
         // Let's dangle the data so that it remains available to the main thread to avoid undefined behavior
         std::mem::forget(evolution);
      });

      Some(thread_handle)
   }


   fn save_compressed_data(path: &std::path::Path, data: &Evolution, tick: usize) -> anyhow::Result<(), FileError> {

      // 1. Create a file
      let file = File::create(path).map_err(FileError::Io)?;
      
      // 2. Wrap it in a BufWriter to speed up disk access
      let buf_writer = BufWriter::new(file);
      
      // 3. Add a Zstd compression layer (compression levels from 1 to 21, 3 is standard)
      let mut encoder = zstd::Encoder::new(buf_writer, 3).map_err(FileError::Io)?;
      
      // 4. Bincode writes directly to the encoder
      // The serialize_into method is more efficient than serialize, as it doesn't create an intermediate Vec<u8>
      let version = 2u8;
      bincode::serialize_into(&mut encoder, &version).map_err(FileError::Bincode)?;
      bincode::serialize_into(&mut encoder, &tick).map_err(FileError::Bincode)?;
      bincode::serialize_into(&mut encoder, &data.elements).map_err(FileError::Bincode)?;
      bincode::serialize_into(&mut encoder, &data.animals).map_err(FileError::Bincode)?;
      encoder.finish().map_err(FileError::Io)?;
      Ok(())
   }


   fn load_compressed_data(path: &std::path::Path) -> anyhow::Result<(ElementsSheets, AnimalsSheet, usize), FileError> {

      // Check if file exists
      if !path.exists() {
         return Err(FileError::NotFound);
      }

      let file = File::open(path).map_err(FileError::Io)?;
      
      let buf_reader = BufReader::new(file);
      
      let mut decoder = zstd::Decoder::new(buf_reader).map_err(FileError::Io)?;
      
      let version: u8 = bincode::deserialize_from(&mut decoder).map_err(FileError::Bincode)?;
      let tick: usize = match version {
         1 => { 0usize } // In version 1, the tick was not saved, so we start from 0
         2 => {
            // Read model time
            bincode::deserialize_from(&mut decoder).map_err(FileError::Bincode)?
         }
         _ => return Err(FileError::Version(version)),
      };

      let elements = bincode::deserialize_from(&mut decoder).map_err(FileError::Bincode)?;
      let animals = bincode::deserialize_from(&mut decoder).map_err(FileError::Bincode)?;

      Ok((elements, animals, tick))
   }


   // Return color for organism, taking into account visibility filters
   fn get_animal_color(&self, o: &Organism) -> Option<iced::Color> {

      // The organism is alive or we show and the dead too
      if self.show_dead || o.alive() {

         // The color of autotrophs is determined by the reaction, for heterotrophs it is predetermined
         match o.nutrition_mode() {
            NutritionMode::Heterotroph => {
               // The color of heterotrophs
               if self.show_heterotrophs {
                  Some(self.env.heterotroph_color)
               } else {
                  None
               }
            }
            NutritionMode::Autotroph => {
               // The color of autotrophs
               let reaction_index = o.reaction_index();
               let visible = self.vis_reac_indexes[reaction_index];

               if visible {
                  let reaction = self.ui_reactions.get(reaction_index);
                  Some(reaction.color)
               } else {
                  None
               }
            }
         }
      } else {
            None
      }
   }


   // Return dot at display position
   // The world must be continuous, the first point goes to the right (or bottom) of the last point again
   pub fn dot(&self, x: isize, y: isize) -> Dot {
      let s = &self.size();

      let x = x.rem_euclid(s.x as isize) as usize;
      let y = y.rem_euclid(s.y as isize) as usize;

      // Corresponding bit of the world
      let serial_bit = self.size().serial(x, y);
      let energy = self.ptr_elements.get(0, serial_bit);

      // Find the dot color among animals
      let mut stack = self.ptr_animals.get_stack(serial_bit);

      // Let's take out the color definition code - determine animal with visible reaction and alive or not
      let animal_color = stack
      .find_map(|o| {
         o.as_ref().and_then(|o| self.get_animal_color(o))
      });

      let mut color = if let Some(color) = animal_color {
         color
      } else {
         // Among elements color determines the element with non-zero amount among visible (non-filtered)
         let color = self.vis_elem_indexes.iter()
         .enumerate()
         .find_map(|(item_index, visible)| {
            if *visible && self.ptr_elements.get(item_index, serial_bit) > 0 {
               Some(self.ui_elements[item_index].color)
            } else {
               None
            }
         });
         
         if let Some(color) = color {
            color
         } else {
            iced::Color::BLACK
         }
      };

      // Adjust color to energy
      color.a = energy as f32 / 100.0;

      Dot{x, y, color,}
   }


   // Text to describe a point with a size constraint
   pub fn description(&self, dot: &Dot, max_lines: usize, delimiter: char) -> String {

      // Underlying bit serial number for dot
      let serial_bit = self.size().serial(dot.x, dot.y);

      // Current time
      let now = self.ticks_elapsed();

      let mut remaining_lines = max_lines;

      // Collect info among animals
      let stack = self.ptr_animals.get_stack(serial_bit);

      // Include animals if color is set
      let filtered_animals = stack.into_iter().flatten()
      .filter(|o| self.get_animal_color(o).is_some());

      // Animal world
      let animal_desc = filtered_animals
      .take(max_lines)
      .fold(String::default(), |acc, o| {
         // After death, the date of birth contains the age at death
         let age = if o.alive() { now.saturating_sub(o.birthday) } else { o.birthday };
         
         // Decrease max lines (side effect)
         remaining_lines -= 1;

         format!("{}[{}۩ {}]{}", acc, age, o, delimiter)
      });

      // Inanimal world
      self.vis_elem_indexes.iter()
      .enumerate()
      .filter(|(_index, vis)| **vis)
      .take(remaining_lines)
      .fold(animal_desc, |acc, (vis_index, _)| {
         format!("{}{}: {}{}", acc, self.ui_elements[vis_index].name, self.ptr_elements.get(vis_index, serial_bit), delimiter)
      })
   }


   // Pause/resume evolutuon thread
   pub fn toggle_run(&self) {
      // Receive signal from thread
      let state: ThreadMode = self.mode.load(Ordering::Acquire).into();
      let state = match state {
         ThreadMode::Run => ThreadMode::Pause,
         _ => ThreadMode::Run,
      };

      // Transfer new signal to thread
      self.mode.store(state as u8, Ordering::Release);
   }


   // Finish thread
   pub fn shutdown(&mut self) {
      let state = ThreadMode::Shutdown;
      self.mode.store(state as u8, Ordering::Release);
      let handle = self.thread_handle.take();
      if let Some(handle) = handle {
         handle.join().expect("world::shutdown error on join");
      }
   }


   // Returns model time - a number ticks elapsed from beginning
   pub fn ticks_elapsed(&self) -> usize {
      self.ticks_elapsed.load(Ordering::Relaxed)
   }


   pub fn size(&self) -> Size {
      self.env.world_size
   }


   pub fn date(&self) -> (usize, usize) {
      let now = self.ticks_elapsed();
      Environment::date(now)
   }


   fn await_for_complete(&self, wait_while: ThreadMode) -> Result<(), String> {
      let timeout = Duration::from_secs(5);
      let sleep_time = Duration::from_millis(100);
      let now = Instant::now();

      while now.elapsed() < timeout {
         let state: ThreadMode = self.mode.load(Ordering::Acquire).into();
         if state != wait_while {
            return Ok(());
         }
         std::thread::sleep(sleep_time);
      }
      Err("Timeout waiting for thread to reach state".into())
   }


   pub fn await_for_save_complete(&self) {
      self.await_for_complete(ThreadMode::Save).expect("world::await_for_save_complete timeout");
   }


   pub fn save(&self) {
      // Send signal to thread to save
      let state = ThreadMode::Save;
      self.mode.store(state as u8, Ordering::Release);
   }


   pub fn busy(&self) -> bool {
      let state: ThreadMode = self.mode.load(Ordering::Relaxed).into();
      state == ThreadMode::Save || state == ThreadMode::Load
   }


   // Indicate loading
   pub fn set_loading(&mut self) {
      self.mode.store(ThreadMode::Load as u8, Ordering::Release);
   }
}


impl Drop for World {
   fn drop(&mut self) {
      self.shutdown();
   }
}


#[allow(dead_code)]
#[derive(Debug)]
enum FileError {
   NotFound,
   Io(std::io::Error),
   Bincode(Box<bincode::ErrorKind>),
   Version(u8),
}