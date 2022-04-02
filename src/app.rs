use std::fmt::Debug;
use std::mem::{replace, take};
use std::ops::Deref;
use std::time::{Duration};
use glam::Vec2;
use instant::Instant;
use crate::opengl::Context;

pub type GlowContext = glow::Context;
pub type Result<T> = anyhow::Result<T>;

#[derive(Debug, Copy, Clone)]
pub enum Event {
    Draw(Duration),
    Resize(u32, u32),
    Click(Vec2),
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
    last_update: Instant
}

impl<G: Game, A: AppContext> Application<G, A> {
    pub fn new() -> Result<Self> {
        let bundle = G::Bundle::new()?;
        Ok(Self {
            state: ApplicationState::Suspended(bundle),
            screen_size: (100, 100),
            last_update: Instant::now()
        })
    }

    pub fn resume(&mut self, ctx_func: impl FnOnce() -> Result<A>) {
        self.state = match take(&mut self.state) {
            ApplicationState::Suspended(bundle) => match ctx_func() {
                Ok(ctx) => {
                    self.screen_size = ctx.screen_size();
                    self.last_update = Instant::now();
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

    pub fn on_click(&mut self, x: f32, y: f32) {
        let (width, height) = self.screen_size;
        let pos = Vec2::new(x / width as f32, 1.0 - y / height as f32);
        self.call_event(Event::Click(pos))
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

