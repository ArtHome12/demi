/* ===============================================================================
Simulation of the evolution of the animal world.
Project controls.
13 Feb 2021.
----------------------------------------------------------------------------
Licensed under the terms of the GPL version 3.
http://www.gnu.org/licenses/gpl-3.0.html
Copyright (c) 2013-2023 by Artem Khomenko _mag12@yahoo.com.
=============================================================================== */

use iced::{Alignment, Element, Length, };
use iced::widget::{checkbox, row, button, container, };

use crate::resources::*;

pub struct Controls {
   run: bool, // run or pause calculation
   pub autosave: bool, // autosave project on exit
   autorun: bool, // autostart evaluations on load
   pub illuminate: bool, // toggle illuminations only from the sun or total
   pub show_filter: bool, // panel at the left side with elements filter
   res: Resources,
}

impl Controls {

   pub fn new(res: Resources) -> Self {
         Self {
         run: false,
         autosave: false,
         autorun: false,
         illuminate: false,
         show_filter: false,
         res,
      }
   }

   pub fn update(&mut self, message: Message) {
      match message {
         Message::ToggleIllumination => self.illuminate = !self.illuminate,
         Message::ToggleFilter => self.show_filter = !self.show_filter,
         Message::ToggleRun => self.run = !self.run,
         Message::ToggleAutosave(checked) => self.autosave = checked,
         Message::ToggleAutorun(checked) => self.autorun = checked,
         _ => ()
      }
   }

   pub fn view(
      &self,
   ) -> Element<'_, Message> {

      container(row![
         // Toggle illuminate
         button(
            self.res.image(if self.illuminate {Images::IlluminateOn} else {Images::IlluminateOff})
         )
         .width(Length::Fixed(24.0 + 20.0))  // fixed size (+padding) prevents flickering due to button resizing on image change
         .height(Length::Fixed(24.0 + 10.0))
         .style(button::secondary)
         .on_press(Message::ToggleIllumination),

         // Show or hide filter panel
         button(
            self.res.image(if self.show_filter {Images::ShowFilter} else {Images::HideFilter}),
         )
         .width(Length::Fixed(24.0 + 20.0))
         .height(Length::Fixed(24.0 + 10.0))
         .style(button::secondary)
         .on_press(Message::ToggleFilter),

         // Pause or run
         button(
            self.res.image(if self.run {Images::ModelPlay} else {Images::ModelPause}),
         )
         .width(Length::Fixed(24.0 + 20.0))
         .height(Length::Fixed(24.0 + 10.0))
         .style(button::secondary)
         .on_press(Message::ToggleRun),

         // Project commands
         button(self.res.image(Images::New))
         .width(Length::Fixed(24.0 + 20.0))
         .height(Length::Fixed(24.0 + 10.0))
         .style(button::secondary)
         .on_press(Message::New),

         button(self.res.image(Images::Load))
         .width(Length::Fixed(24.0 + 20.0))
         .height(Length::Fixed(24.0 + 10.0))
         .style(button::secondary)
         .on_press(Message::Load),

         button(self.res.image(Images::Save))
         .width(Length::Fixed(24.0 + 20.0))
         .height(Length::Fixed(24.0 + 10.0))
         .style(button::secondary)
         .on_press(Message::Save),

         checkbox(self.autosave)
         .label("Autosave")
         .size(16)
         .spacing(5)
         .text_size(16)
         .on_toggle(Message::ToggleAutosave),

         checkbox(self.autorun)
         .label("Autorun")
         .size(16)
         .spacing(5)
         .text_size(16)
         .on_toggle(Message::ToggleAutorun),
      ].spacing(10)
         .align_y(Alignment::Center)
      )
      .width(Length::Fill)
      .padding(10)
      .style(container::bordered_box)
      .into()
   }
}

#[derive(Debug, Clone)]
pub enum Message {
   ToggleIllumination,
   ToggleFilter,
   New,
   Load,
   Save,
   ToggleRun,
   ToggleAutosave(bool),
   ToggleAutorun(bool),
}

