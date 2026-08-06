/* ===============================================================================
Simulation of the evolution of the animal world.
Control for filtering displayed elements.
09 Apr 2021.
----------------------------------------------------------------------------
Licensed under the terms of the GPL version 3.
http://www.gnu.org/licenses/gpl-3.0.html
Copyright (c) 2013-2023 by Artem Khomenko _mag12@yahoo.com.
=============================================================================== */

use std::{rc::Rc, cell::RefCell, };

use iced::{Element, Length};
use iced::widget::{scrollable,
   container, column, Column, checkbox, text,
};


use crate::world::World;


#[derive(Debug, Clone, Copy)]
pub enum Message {
   ItemToggledElement(usize, bool), // index, checked
   ItemToggledAnimal(usize, bool), // index, checked
   ItemToggledDead(bool), // checked
   ItemToggledHeterotrophs(bool), // checked
}


pub struct Controls {
   // scroll: scrollable::State,
   world: Rc<RefCell<World>>,
}

impl Controls {
   pub fn new(world: Rc<RefCell<World>>) -> Self {
      Self{
         // scroll: scrollable::State::new(),
         world,
      }
   }

   pub fn update(&mut self, message: Message) {
      // Storage for visible items
      let world = &mut self.world.borrow_mut();

      match message {
         Message::ItemToggledElement(index, checked) => world.vis_elem_indexes[index] = checked,
         Message::ItemToggledAnimal(index, checked) => world.vis_reac_indexes[index] = checked,
         Message::ItemToggledDead(checked) => world.show_dead = checked,
         Message::ItemToggledHeterotrophs(checked) => world.show_heterotrophs = checked,
      }
  }

   pub fn view(&self) -> Element<'_, Message> {

      let world = &self.world.borrow();
      
      let elements_check_boxes = world.ui_elements
      .iter()
      .enumerate()
      .fold(Column::new().spacing(10), |column, (index, item)| {
         column.push(
            checkbox(world.vis_elem_indexes[index])
            .label(item.name.clone())
            .on_toggle(move |b| Message::ItemToggledElement(index, b))
            .text_size(16)
            .size(16)
         )
      });

      let dead_check_box = checkbox(world.show_dead)
      .label("Include dead")
      .on_toggle(Message::ItemToggledDead)
      .text_size(16)
      .size(16);

      let heterotrophs_check_box = checkbox(world.show_heterotrophs)
      .label("Heterotrophs")
      .on_toggle(Message::ItemToggledHeterotrophs)
      .text_size(16)
      .size(16);

      let animal_check_boxes = world.ui_reactions
      .iter()
      .enumerate()
      .fold(Column::new().spacing(10), |column, (index, item)| {
         // Reaction name
         column.push(
            checkbox(world.vis_reac_indexes[index])
            .label(item.name.clone())
            .on_toggle(move |b| Message::ItemToggledAnimal(index, b))
            .text_size(16)
            .size(16)
         )
      });
     
      let content = scrollable(
         column![
            text("Elements to show:"),
            elements_check_boxes,
            text("Animals to show:"),
            dead_check_box,
            heterotrophs_check_box,
            animal_check_boxes
         ].spacing(8)
      );

      container(content)
         .width(Length::Fill)
         .height(Length::Fill)
         .padding(5)
         .style(container::bordered_box)
         .into()
  }
}
