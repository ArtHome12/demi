/* ===============================================================================
Simulation of the evolution of the animal world.
Main window.
25 Jan 2021.
----------------------------------------------------------------------------
Licensed under the terms of the GPL version 3.
http://www.gnu.org/licenses/gpl-3.0.html
Copyright (c) 2013-2023 by Artem Khomenko _mag12@yahoo.com.
=============================================================================== */

mod project_controls;
mod grid;
mod dot;
mod world;
mod project;
mod evolution;
mod update_rate;
mod geom;
mod environment;
mod resources;
mod filter_control;
mod reactions;
mod genes;
mod organism;

use grid::Grid;
use iced::{Element, Subscription, Task, Window, window, window::Id, Theme, mouse};
use iced::widget::{column, PaneGrid, pane_grid, pane_grid::Axis, mouse_area,};
use iced::window::icon;

use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{rc::Rc, cell::RefCell, };



// #[tokio::main]
pub fn main() -> iced::Result {

   let icon_path = Path::new("./res/icon32.jpg");
   let icon = icon::from_file(icon_path).ok();
   let settings = iced::window::Settings {
      icon,
      ..iced::window::Settings::default()
   };

   iced::application(Demi::new, Demi::update, Demi::view)
      .subscription(Demi::subscription)
      .exit_on_close_request(false)
      .antialiasing(true)
      .theme(Theme::Nightfly)
      .window(settings)
      .title("Demi")
      .run()
}


#[derive(Clone)]
enum Message {
   StartupAction,
   ProjectMessage(project_controls::Message), // Messages from project controls at top line
   GridMessage(grid::Message), // Messages from grid
   FilterMessage(filter_control::Message), // Messages from filter pane at right

   Refresh(Instant), // called about 30 times per second for screen refresh

   ResizeEvent((Id, iced::Size)), // Message from main window
   Resized(pane_grid::ResizeEvent), // Message from panes

   // Commands to execute
   ShowFilter(bool),
   Illuminate(bool),
   ToggleRun,
   CloseEvent(Id),
   SaveWorld,
   NewWorld,
   SelectFile, // Open file dialog
   OpenFile(Option<PathBuf>),
   FileOpened(Arc<UnblownDemi>), // after new world created
}


struct Demi {
   world: Rc<RefCell<world::World>>,
   controls: project_controls::Controls,
   panes: pane_grid::State<PaneState>,
   grid_pane: pane_grid::Pane,
   filter_pane: Option<pane_grid::Pane>,
   grid_pane_ratio: f32, // Width of grid when filter visible too
   last_one_second_time: Instant,   // for FPS/TPS evaluations
}


// Unblown version of Demi for async world creation
#[derive(Debug)]
struct UnblownDemi {
   world: world::World,
   prev_size: grid::PrevState,
}


impl Demi {
   fn new() -> (Self, Task<Message>) {
      // Project contains info for create model
      let filename = PathBuf::from("./demi.toml");
      let project = project::Project::new(&filename);

      // World contains model and manage its evaluation
      let world = world::World::new(project.clone());
      let res = Self::init(world);
      (res, Task::done(Message::StartupAction))
   }

   fn update(&mut self, message: Message) -> Task<Message> {
      match message {
         Message::StartupAction => {
            // Load last project
            // self.world.borrow_mut().load_last_project();
            Task::none()
         }

         Message::ProjectMessage(message) => {

            // Reflecting the interface change immediately
            self.controls.update(message.clone());

            // Translate from project message to own and create new command
            match message {
               project_controls::Message::ToggleRun => Task::done(Message::ToggleRun),
               project_controls::Message::ToggleIllumination => Task::done(Message::Illuminate(self.controls.illuminate)),
               project_controls::Message::ToggleFilter => Task::done(Message::ShowFilter(self.controls.show_filter)),
               project_controls::Message::Save => Task::done(Message::SaveWorld),
               project_controls::Message::New => Task::done(Message::NewWorld),
               project_controls::Message::Load => Task::done(Message::SelectFile),
               _ => Task::none(),
            }
         }

         Message::GridMessage(message) => {
            self.grid_mut()
               .update(message)
               .map(| msg | Message::GridMessage(msg))
         }

         Message::FilterMessage(message) => {
            self.filter_mut().update(message);

            let grid_cmd = grid::Message::FilterChanged;
            let cmd = Message::GridMessage(grid_cmd);
            Task::done(cmd)
         }

         Message::Refresh(ts) => {
            // Update rates once a second
            let one_second_passed = ts.duration_since(self.last_one_second_time) >= Duration::new(1, 0);
            if one_second_passed {
               self.last_one_second_time = ts;
            }

            let grid_cmd = grid::Message::ClockChime(one_second_passed);
            let cmd = Message::GridMessage(grid_cmd);
            Task::done(cmd)
         }

         Message::ResizeEvent((_id, size)) => {
            let cmd = Message::GridMessage(grid::Message::Resized(size));
            Task::done(cmd)
         }

         Message::Resized(pane_grid::ResizeEvent { split, ratio }) => {
            self.grid_pane_ratio = ratio;
            self.panes.resize(split, ratio);
            Task::none()
         }

         Message::ShowFilter(show) => {
            if show {
               // Construct filter pane. Maybe it should be converted to an asynchronous version.
               let w = self.world.clone();
               let content = filter_control::Controls::new(w);
               let content = PaneContent::Filter(content);
               let content = PaneState::new(content);
               let pair = self.panes.split(Axis::Vertical, self.grid_pane, content);

               // Store pane and restore ratio
               self.filter_pane = if let Some(pair) = pair {
                  self.panes.resize(pair.1, self.grid_pane_ratio);
                  Some(pair.0)
               } else {
                  None
               };

            } else {
               // Close filter
               let filter_pane = self.filter_pane.expect("update() Attempt to close non-existent filter_pane");
               self.panes.close(filter_pane);
               self.filter_pane = None;
            }
            Task::none()
         }

         Message::Illuminate(is_on) => {
            let grid_cmd = grid::Message::Illumination(is_on);
            let cmd = Message::GridMessage(grid_cmd);
            Task::done(cmd)
         }

         Message::ToggleRun => {
            self.world.borrow().toggle_run();
            Task::none()
         }

         Message::CloseEvent(_id) => {
            self.world.borrow_mut().shutdown();
            return iced::exit()
         }

         Message::SaveWorld => {
            self.world.borrow().save();
            Task::none()
         }

         Message::NewWorld => {

            let template = PathBuf::from("./demi.toml");
            let cmd = Message::OpenFile(Some(template));
            Task::done(cmd)
         }

         Message::SelectFile => {
            window::oldest()
               .and_then(|id| window::run(id, open_file_dialog))
               .then(Task::future)
               .map(Message::OpenFile)
         }

         Message::OpenFile(pathbuf) => {

            let Some(pathbuf) = pathbuf else {
               // File dialog was closed without picking a file
               return Task::none();
            };

            // Keep the same size
            let prev_size = self.grid_mut().prev_state();

            let future = async move {
               let project = project::Project::new(&pathbuf);
               let world = world::World::new(project);
               Arc::new(UnblownDemi {
                  world,
                  prev_size,
               })
            };

            Task::perform(future, Message::FileOpened)
         }

         Message::FileOpened(unblown) => {
            let unblown = Arc::try_unwrap(unblown).expect("FileOpened() Failed to unwrap Arc");
            *self = Self::init(unblown.world);
            self.grid_mut().restore_state(unblown.prev_size);
            Task::none()
         }
      }
   }


   fn init(world: world::World) -> Self{
      let world = Rc::new(RefCell::new(world));
      let controls = project_controls::Controls::new(resources::Resources::new("./res"));

      let grid_world = Rc::clone(&world);
      let grid = Grid::new(grid_world);
      let (panes, grid_pane) = pane_grid::State::new(PaneState::new(PaneContent::Grid(grid)));

      let res = Self {
         world,
         controls,
         panes,
         grid_pane,
         filter_pane: None,
         grid_pane_ratio: 0.8,
         last_one_second_time: Instant::now(),
      };
      res
   }


   fn subscription(&self) -> Subscription<Message> {
      let subs = vec![window::frames().map(Message::Refresh),
         window::close_requests().map(Message::CloseEvent),
         window::resize_events().map(|(id, size)| Message::ResizeEvent((id, size))),
      ];
      Subscription::batch(subs)
   }

   fn view(&self) -> Element<'_, Message> {

      // Place project controls at top line
      let controls = self.controls.view().map(Message::ProjectMessage);

      // Client area contains grid with the world's dots and sometimes filter for showing elements
      let pane_grid = PaneGrid::new(&self.panes, |_id, pane, _is_maximized| {
         pane_grid::Content::new(pane.view())
      })
      .on_resize(10, Message::Resized);

      let widget = column![
         controls,
         pane_grid,
      ];

      // Set the cursor if necessary
      let busy = self.world.borrow().busy();
      if busy {
         mouse_area(widget)
         .interaction(mouse::Interaction::Progress)
         .into()
      } else {
         widget
         .into()
      }
   }

   fn grid_mut(&mut self) -> &mut Grid {
      match self.panes.get_mut(self.grid_pane).unwrap().content {
         PaneContent::Grid(ref mut grid) => grid,
         _ => panic!("grid_mut() Pane is not a grid"),
      }
   }


   fn filter_mut(&mut self) -> &mut filter_control::Controls {
      let pane = self.filter_pane.expect("filter_mut() Attempt to get_mut non-existent filter_pane");
      match self.panes.get_mut(pane).unwrap().content {
         PaneContent::Filter(ref mut elements) => elements,
         _ => panic!("filter_mut() Pane is not a filter_control::Controls"),
      }
   }

}

enum PaneContent {
   Grid(Grid),
   Filter(filter_control::Controls),
}

struct PaneState {
   pub content: PaneContent,
}

impl PaneState {
   fn new(content: PaneContent) -> Self {
       PaneState {
            content,
       }
   }
   
   fn view(&self) -> Element<'_, Message> {
      match &self.content {
         PaneContent::Grid(grid) => grid.view().map(move |message| Message::GridMessage(message)),
         PaneContent::Filter(filter) => filter.view().map(move |message| Message::FilterMessage(message)),
      }
   }
}

#[derive(Debug, Clone)]
pub enum Error {
    DialogClosed,
    IoError(io::ErrorKind),
}


fn open_file_dialog(
   window: &dyn Window,
) -> impl std::future::Future<Output = Option<PathBuf>> {
   let dialog = rfd::AsyncFileDialog::new()
      .set_title("Open a project...")
      .add_filter("demi projects", &["toml"])
      .add_filter("all files", &["*"])
      .set_parent(&window);

   async move {
      let picked_file = dialog.pick_file().await;
      picked_file.map(|file| file.into())
   }
}

