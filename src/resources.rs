/* ===============================================================================
Simulation of the evolution of the animal world.
Graphic resources.
30 Mar 2021.
----------------------------------------------------------------------------
Licensed under the terms of the GPL version 3.
http://www.gnu.org/licenses/gpl-3.0.html
Copyright (c) 2013-2022 by Artem Khomenko _mag12@yahoo.com.
=============================================================================== */

use iced::widget::{Image, image::Handle, };

pub enum Images {
   IlluminateOn,
   IlluminateOff,
   ShowFilter,
   HideFilter,
   ModelPlay,
   ModelPause,
   New,
   Load,
   Save,
}

pub struct Resources {
   illuminate_on: Handle,
   illuminate_off: Handle,
   show_filter: Handle,
   hide_filter: Handle,
   model_play: Handle,
   model_pause: Handle,
   new: Handle,
   load: Handle,
   save: Handle,
}

impl Resources {
   #[must_use]
   pub fn new(dir: &str) -> Self {
      Self {
         illuminate_on: Handle::from_path(String::from(dir) + "/illuminate_on.png"),
         illuminate_off: Handle::from_path(String::from(dir) + "/illuminate_off.png"),
         show_filter: Handle::from_path(String::from(dir) + "/show_filter.png"),
         hide_filter: Handle::from_path(String::from(dir) + "/hide_filter.png"),
         model_play: Handle::from_path(String::from(dir) + "/model_play.png"),
         model_pause: Handle::from_path(String::from(dir) + "/model_pause.png"),
         new: Handle::from_path(String::from(dir) + "/new.png"),
         load: Handle::from_path(String::from(dir) + "/load.png"),
         save: Handle::from_path(String::from(dir) + "/save.png"),
      }
   }

   pub fn image(&self, image: Images) -> Image<Handle> {
      let handle = match image {
         Images::IlluminateOn => self.illuminate_on.clone(),
         Images::IlluminateOff => self.illuminate_off.clone(),
         Images::ShowFilter => self.show_filter.clone(),
         Images::HideFilter => self.hide_filter.clone(),
         Images::ModelPlay => self.model_play.clone(),
         Images::ModelPause => self.model_pause.clone(),
         Images::New => self.new.clone(),
         Images::Load => self.load.clone(),
         Images::Save => self.save.clone(),
      };
      Image::new(handle)
   }
}