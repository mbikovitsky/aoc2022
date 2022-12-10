use std::str::FromStr;

use anyhow::{bail, Result};
use aoc2022::util::input_lines;
use itertools::Itertools;

fn main() -> Result<()> {
    let program = parse_input()?;

    let mut cpu = Cpu::new(program.iter().copied());
    let mut signal_strength_sum = 0;
    for cycle_count in (19..=219).step_by(40) {
        while cpu.cycle_count() != cycle_count {
            assert!(cpu.execute_cycle());
        }

        // We're actually interested in the X value *during* the next cycle.
        // It's the same value, but we need to adjust the computation a little.
        let signal_strength = (cpu.cycle_count() + 1) as i64 * cpu.x_register() as i64;
        signal_strength_sum += signal_strength;
    }
    dbg!(signal_strength_sum);

    let mut system = System::new(Cpu::new(program));
    system.run();
    for row in 0..System::SCREEN_HEIGHT {
        for column in 0..System::SCREEN_WIDTH {
            if system.pixel_at(row, column) {
                print!("#")
            } else {
                print!(".")
            }
        }
        println!();
    }

    Ok(())
}

fn parse_input() -> Result<Vec<Instruction>> {
    input_lines()?
        .into_iter()
        .map(|line| line.parse())
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Instruction {
    AddX(i32),
    NoOp,
}

impl FromStr for Instruction {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s == "noop" {
            Ok(Self::NoOp)
        } else if let Some(("addx", operand)) = s.split_whitespace().collect_tuple() {
            let operand: i32 = operand.parse()?;
            Ok(Self::AddX(operand))
        } else {
            bail!("Invalid instruction {}", s)
        }
    }
}

#[derive(Debug, Clone)]
struct Cpu {
    program: Vec<Instruction>,
    cycle_count: u32,
    x_register: i32,
    program_counter: usize,
    state: CpuState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CpuState {
    Idle,
    ExecutingAddX,
}

impl Cpu {
    pub fn new(program: impl IntoIterator<Item = Instruction>) -> Self {
        Self {
            program: program.into_iter().collect(),
            cycle_count: 0,
            x_register: 1,
            program_counter: 0,
            state: CpuState::Idle,
        }
    }

    pub fn cycle_count(&self) -> u32 {
        self.cycle_count
    }

    pub fn x_register(&self) -> i32 {
        self.x_register
    }

    pub fn execute_cycle(&mut self) -> bool {
        if self.program_counter >= self.program.len() {
            return false;
        }

        match self.state {
            CpuState::Idle => {
                // Idle state. Read the next instruction.
                let instruction = self.program[self.program_counter];
                match instruction {
                    Instruction::AddX(_) => self.state = CpuState::ExecutingAddX,
                    Instruction::NoOp => self.state = CpuState::Idle,
                }
            }
            CpuState::ExecutingAddX => {
                match self.program[self.program_counter] {
                    Instruction::AddX(argument) => self.x_register += argument,
                    _ => unreachable!(),
                }
                self.state = CpuState::Idle;
            }
        }

        // If after execution we're in the idle state, then the instruction
        // is done. Increment PC.
        if let CpuState::Idle = self.state {
            self.program_counter += 1
        }

        self.cycle_count += 1;

        true
    }
}

#[derive(Debug, Clone)]
struct System {
    cpu: Cpu,
    screen: [bool; Self::SCREEN_WIDTH * Self::SCREEN_HEIGHT],
}

impl System {
    const SCREEN_WIDTH: usize = 40;
    const SCREEN_HEIGHT: usize = 6;

    pub fn new(cpu: Cpu) -> Self {
        Self {
            cpu,
            screen: [false; Self::SCREEN_WIDTH * Self::SCREEN_HEIGHT],
        }
    }

    pub fn pixel_at(&self, row: usize, column: usize) -> bool {
        assert!(row < Self::SCREEN_HEIGHT);
        assert!(column < Self::SCREEN_WIDTH);
        self.screen[row * Self::SCREEN_WIDTH + column]
    }

    pub fn run(&mut self) {
        loop {
            // During the next cycle, the beam will draw the pixel at this position.
            let horizontal_position: usize = self.cpu.cycle_count() as usize % Self::SCREEN_WIDTH;
            let vertical_position: usize =
                (self.cpu.cycle_count() as usize / Self::SCREEN_WIDTH) % Self::SCREEN_HEIGHT;

            let sprite_middle = self.cpu.x_register();
            let sprite_left = sprite_middle - 1;
            let sprite_right = sprite_middle + 1;

            let pixel_index: usize = vertical_position * Self::SCREEN_WIDTH + horizontal_position;

            if sprite_left == horizontal_position.try_into().unwrap()
                || sprite_right == horizontal_position.try_into().unwrap()
                || sprite_middle == horizontal_position.try_into().unwrap()
            {
                self.screen[pixel_index] = true;
            } else {
                self.screen[pixel_index] = false;
            }

            if !self.cpu.execute_cycle() {
                break;
            }
        }
    }
}
