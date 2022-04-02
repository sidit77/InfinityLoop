use std::fmt::Debug;
use std::mem::{replace, take};
use std::ops::Deref;
use std::time::{Duration};
use glam::Vec2;
use instant::Instant;
use crate::{log_assert, log_unreachable};
use crate::opengl::Context;

pub type GlowContext = glow::Context;
pub type Result<T> = anyhow::Result<T>;

#[derive(Debug, Copy, Clone)]
pub enum Event {
    Draw(Duration),
    Resize(u32, u32),
    Click(Vec2),
    Drag(Vec2),
    Zoom(Vec2, f32),
}

enum ApplicationState<G: Game, A: AppContext> {
    Active{
        game: G,
        ctx: A,
        should_redraw: bool
    },
    Suspended(G::Bundle),
    Invalid
}

impl<G: Game, A: AppContext> Default for ApplicationState<G, A> {
    fn default() -> Self {
        Self::Invalid
    }
}

pub trait AppContext: Deref<Target = Context> {

    fn gl(&self) -> &Context;

    fn screen_size(&self) -> (u32, u32);

    fn screen_width(&self) -> u32 {
        self.screen_size().0
    }

    fn screen_height(&self) -> u32 {
        self.screen_size().1
    }
}


pub struct Application<G: Game, A: AppContext> {
    state: ApplicationState<G, A>,
    screen_size: (u32, u32),
    last_update: Instant,
    input_state: InputState,
    touches: TouchMap
}

impl<G: Game, A: AppContext> Application<G, A> {
    pub fn new() -> Result<Self> {
        let bundle = G::Bundle::new()?;
        Ok(Self {
            state: ApplicationState::Suspended(bundle),
            screen_size: (100, 100),
            last_update: Instant::now(),
            input_state: InputState::Up,
            touches: TouchMap::new()
        })
    }

    pub fn resume(&mut self, ctx_func: impl FnOnce() -> Result<A>) {
        self.state = match take(&mut self.state) {
            ApplicationState::Suspended(bundle) => match ctx_func() {
                Ok(ctx) => {
                    self.screen_size = ctx.screen_size();
                    self.last_update = Instant::now();
                    self.input_state = InputState::Up;
                    self.touches = TouchMap::new();
                    match G::resume(&ctx, bundle.clone()) {
                        Ok(game) => {
                            log::info!("Resumed app");
                            ApplicationState::Active{
                                game,
                                ctx,
                                should_redraw: true
                            }
                        },
                        Err(err) => {
                            log::error!("Can't resume application:\n{}", err);
                            ApplicationState::Suspended(bundle)
                        }
                    }
                },
                Err(err) => {
                    log::error!("Can't create context:\n{}", err);
                    ApplicationState::Suspended(bundle)
                }
            },
            state => state
        }
    }

    pub fn suspend(&mut self) -> Option<A> {
        let (ctx, state) = match take(&mut self.state) {
            ApplicationState::Active{ game, ctx, ..} => {
                log::info!("Suspended app");
                let bundle = game.suspend(&ctx);
                (Some(ctx), ApplicationState::Suspended(bundle))
            },
            state => (None, state)
        };
        self.state = state;
        ctx
    }

    pub fn set_screen_size(&mut self, screen_size: (u32, u32)) {
        self.screen_size = screen_size;
        self.call_event(Event::Resize(screen_size.0, screen_size.1));
    }

    pub fn on_press(&mut self, x: f32, y: f32, id: u64) {
        if self.touches.insert(id, self.normalize(x, y)) {
            match self.input_state {
                InputState::Up => {
                    log_assert!(self.touches.len() == 1);
                    self.input_state = InputState::Click(self.touches.center().unwrap());
                }
                InputState::Click(_) => {
                    log_assert!(self.touches.len() > 1);
                    self.input_state = InputState::Drag(self.touches.center().unwrap());
                }
                InputState::Drag(_) => {
                    self.input_state = InputState::Drag(self.touches.center().unwrap());
                }
            }
        }
    }

    pub fn on_release(&mut self, _x: f32, _y: f32, id: u64) {
        self.touches.remove(id);
        if self.touches.len() == 0 {
            match self.input_state {
                InputState::Up => log_unreachable!(),
                InputState::Click(pos) => self.call_event(Event::Click(pos)),
                InputState::Drag(_) => {}
            }
            self.input_state = InputState::Up;
        } else {
            match self.input_state {
                InputState::Drag(_) => {
                    self.input_state = InputState::Drag(self.touches.center().unwrap());
                }
                _ => log_unreachable!()
            }
        }
    }

    pub fn on_move(&mut self, x: f32, y: f32, id: u64) {
        if self.touches.contains(id) {
            let dist1 = self.touches.distance();
            self.touches.update(id, self.normalize(x, y));
            let npos = self.touches.center().unwrap();
            if let Some(dist1) = dist1 {
                let dist2 = self.touches.distance().unwrap();
                self.call_event(Event::Zoom(npos, (dist2 - dist1) * 30.0));
            }
            match self.input_state {
                InputState::Up => log_unreachable!(),
                InputState::Click(pos) => if pos.distance(npos) > 0.01 {
                    self.input_state = InputState::Drag(npos);
                    self.call_event(Event::Drag(npos - pos));
                }
                InputState::Drag(pos) => {
                    self.input_state = InputState::Drag(npos);
                    self.call_event(Event::Drag(npos - pos));
                }
            }
        }
    }

    pub fn on_mouse_wheel(&mut self, x: f32, y: f32, amt: f32){
        self.call_event(Event::Zoom(self.normalize(x, y), amt))
    }

    fn normalize(&self, x: f32, y: f32) -> Vec2 {
        let (width, height) = self.screen_size;
        Vec2::new(x / width as f32, 1.0 - y / height as f32)
    }

    pub fn redraw(&mut self) {
        let now = Instant::now();
        let delta = now - replace(&mut self.last_update, now);
        self.call_event(Event::Draw(delta))
    }

    pub fn should_redraw(&self) -> bool {
        match self.state {
            ApplicationState::Active { should_redraw, ..} => should_redraw,
            _ => false
        }
    }

    pub fn with_ctx<R: Default>(&self, f: impl FnOnce(&A) -> R) -> R{
        if let ApplicationState::Active{  ctx, ..} = &self.state {
            return f(ctx)
        }
        R::default()
    }

    fn call_event(&mut self, event: Event) {
        if let ApplicationState::Active{ game, ctx, should_redraw} = &mut self.state {
            if ! *should_redraw {
                self.last_update = Instant::now();
            }
            *should_redraw = game.event(ctx, event).unwrap();

        }
    }

}

pub trait Bundle: Clone + Sized {
    fn new() -> Result<Self>;
}

pub trait Game: Sized {
    type Bundle: Bundle;

    fn resume<A: AppContext>(ctx: &A, bundle: Self::Bundle) -> Result<Self>;
    fn suspend<A: AppContext>(self, ctx: &A) -> Self::Bundle;

    fn event<A: AppContext>(&mut self, ctx: &A, event: Event) -> Result<bool>;
}

#[derive(Copy, Clone, Debug)]
enum InputState {
    Up,
    Click(Vec2),
    Drag(Vec2)
}

struct TouchMap {
    touches: [Option<(u64, Vec2)>; 2]
}

impl TouchMap {

    fn new() -> Self {
        Self {
            touches: [None; 2]
        }
    }

    fn insert(&mut self, id: u64, pos: Vec2) -> bool {
        match self.contains(id) {
            true => false,
            false => match self.touches.iter_mut().find(|e|e.is_none()) {
                None => false,
                Some(slot) => {
                    *slot = Some((id, pos));
                    true
                }
            }
        }

    }

    fn remove(&mut self, id: u64) {
        for touch in &mut self.touches {
            if let Some((key, _)) = touch {
                if *key == id {
                    *touch = None;
                }
            }
        }
    }

    fn update(&mut self, id: u64, pos: Vec2) {
        for touch in &mut self.touches {
            if let Some((key, value)) = touch {
                if *key == id {
                    *value = pos;
                }
            }
        }
    }

    fn len(&self) -> usize {
        self.touches.iter()
            .filter_map(|e|*e)
            .count()
    }

    fn contains(&self, id: u64) -> bool {
        self.touches.iter()
            .filter_map(|e|*e)
            .any(|(key, _)| key == id)
    }

    fn center(&self) -> Option<Vec2> {
        let mut sum = Vec2::ZERO;
        let mut count = 0;
        for touch in self.touches {
            if let Some((_, value)) = touch {
                sum += value;
                count += 1;
            }
        }
        if count > 0 {
            Some(sum / count as f32)
        } else {
            None
        }
    }

    fn distance(&self) -> Option<f32> {
        if let Some((_, v1)) = self.touches[0] {
            if let Some((_, v2)) = self.touches[1] {
                return Some(v1.distance(v2))
            }
        }
        None
    }

}
