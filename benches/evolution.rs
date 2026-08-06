/* ===============================================================================
Simulation of the evolution of the animal world.
Benchmarks for evolution module.
06 Aug 2026.
----------------------------------------------------------------------------
Licensed under the terms of the GPL version 3.
http://www.gnu.org/licenses/gpl-3.0.html
Copyright (c) 2013-2026 by Artem Khomenko <_mag12@yahoo.com>.
=============================================================================== */

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use demi::geom::Size;
use demi::environment::Environment;
use demi::organism::{Organism, AnimalsStack, AnimalsSheet};
use demi::genes::{Digestion, NutritionMode, Reproduction};
use demi::state::FSM;

fn benchmark_environment_creation(c: &mut Criterion) {
   let mut group = c.benchmark_group("environment_creation");

   for size in &[10, 20, 50, 100] {
      let world_size = Size { x: *size, y: *size };
      group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
         b.iter(|| {
            black_box(Environment::new(
               black_box(world_size),
               black_box(0.01),
               black_box(0),
               black_box(1),
               black_box(iced::Color::WHITE)
            ));
         });
      });
   }

   group.finish();
}

fn benchmark_organism_creation(c: &mut Criterion) {
   let mut group = c.benchmark_group("organism_creation");

   group.bench_function("basic", |b| {
      let vitality = 1000;
      let digestion = Digestion::new(0, 1, NutritionMode::Autotroph);
      let reproduction = Reproduction::new(3000);
      let fcm = FSM { index: 0, lines: vec![] };

      b.iter(|| {
         black_box(Organism::new(
            black_box(vitality),
            black_box(0),
            black_box(digestion.clone()),
            black_box(reproduction.clone()),
            black_box(fcm.clone())
         ));
      });
   });

   group.finish();
}

fn benchmark_organism_alive(c: &mut Criterion) {
   let mut group = c.benchmark_group("organism_alive");

   let vitality = 1000;
   let digestion = Digestion::new(0, 1, NutritionMode::Autotroph);
   let reproduction = Reproduction::new(3000);
   let fcm = FSM { index: 0, lines: vec![] };
   let organism = Organism::new(vitality, 0, digestion, reproduction, fcm);

   group.bench_function("alive", |b| {
      b.iter(|| {
         black_box(black_box(&organism).alive());
      });
   });

   group.finish();
}

fn benchmark_organism_reproduction(c: &mut Criterion) {
   let mut group = c.benchmark_group("organism_reproduction");

   let vitality = 1000;
   let level = 3000;
   let digestion = Digestion::new(0, 1, NutritionMode::Autotroph);
   let reproduction = Reproduction::new(level);
   let fcm = FSM { index: 0, lines: vec![] };

   group.bench_function("successful", |b| {
      let mut rng = rand::rng();

      b.iter(|| {
         let mut org = Organism::new(vitality, 0, digestion.clone(), reproduction.clone(), fcm.clone());
         org.vitality = level + 1;
         let result = org.reproduction(black_box(&mut rng));
         black_box(result);
      });
   });

   group.bench_function("failed", |b| {
      let mut rng = rand::rng();

      b.iter(|| {
         let mut org = Organism::new(vitality, 0, digestion.clone(), reproduction.clone(), fcm.clone());
         org.vitality = level - 1;
         let result = org.reproduction(black_box(&mut rng));
         black_box(result);
      });
   });

   group.finish();
}

fn benchmark_animals_stack_creation(c: &mut Criterion) {
   let mut group = c.benchmark_group("animals_stack_creation");

   for size in &[10, 50, 100] {
      group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, size| {
         b.iter(|| {
            black_box(AnimalsStack::new(black_box(*size)));
         });
      });
   }

   group.finish();
}

fn benchmark_animals_sheet_creation(c: &mut Criterion) {
   let mut group = c.benchmark_group("animals_sheet_creation");

   for size in &[10, 20, 50] {
      let world_size = Size { x: *size, y: *size };
      let max_serial = world_size.x * world_size.y;

      group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
         b.iter(|| {
            black_box(AnimalsSheet::new(black_box(max_serial), black_box(10)));
         });
      });
   }

   group.finish();
}

fn benchmark_genes_creation(c: &mut Criterion) {
   let mut group = c.benchmark_group("genes_creation");

   group.bench_function("digestion", |b| {
      b.iter(|| {
         black_box(Digestion::new(black_box(0), black_box(10), black_box(NutritionMode::Autotroph)));
      });
   });

   group.bench_function("reproduction", |b| {
      b.iter(|| {
         black_box(Reproduction::new(black_box(1000)));
      });
   });

   group.finish();
}

fn benchmark_genes_clone(c: &mut Criterion) {
   let mut group = c.benchmark_group("genes_clone");

   let digestion = Digestion::new(0, 10, NutritionMode::Autotroph);
   let reproduction = Reproduction::new(1000);

   group.bench_function("digestion", |b| {
      b.iter(|| {
         let _ = black_box(&digestion).clone();
      });
   });

   group.bench_function("reproduction", |b| {
      b.iter(|| {
         let _ = black_box(&reproduction).clone();
      });
   });

   group.finish();
}

fn benchmark_fsm_creation(c: &mut Criterion) {
   let mut group = c.benchmark_group("fsm_creation");

   for size in &[0, 10, 100] {
      group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, size| {
         let lines = vec![demi::state::Line::new(10); *size];
         b.iter(|| {
            black_box(lines.clone());
         });
      });
   }

   group.finish();
}

fn benchmark_line_operations(c: &mut Criterion) {
   let mut group = c.benchmark_group("line_operations");

   let size = 100;
   let mut line = demi::state::Line::new(size);

   group.bench_function("set", |b| {
      b.iter(|| {
         let mut l = line.clone();
         for i in 0..size {
            l.set(i, demi::state::State::On);
         }
         black_box(l);
      });
   });

   group.bench_function("get", |b| {
      for i in 0..size {
         line.set(i, demi::state::State::On);
      }
      b.iter(|| {
         let mut sum = 0;
         for i in 0..size {
            if line.get(i) == demi::state::State::On {
               sum += 1;
            }
         }
         black_box(sum);
      });
   });

   group.finish();
}

criterion_group!(
   benches,
   benchmark_environment_creation,
   benchmark_organism_creation,
   benchmark_organism_alive,
   benchmark_organism_reproduction,
   benchmark_animals_stack_creation,
   benchmark_animals_sheet_creation,
   benchmark_genes_creation,
   benchmark_genes_clone,
   benchmark_fsm_creation,
   benchmark_line_operations
);
criterion_main!(benches);