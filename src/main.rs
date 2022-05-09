mod mathfuncs;

use std::{env, path};
use mathfuncs::{vec_from_angle, generate_random_scoords};

use ggez::{Context, ContextBuilder, GameResult};
use ggez::graphics::{self, Color, DrawParam, Text, Font, PxScale};
use ggez::event::{EventHandler, MouseButton};
use ggez::input::{mouse};
use ggez::audio;
use ggez::audio::SoundSource;
use glam::Vec2;

use rand::prelude::*;
struct Bullet { 
    bullet_image: graphics::Image,
    position: Vec2,
    velocity: Vec2,
    lifetime: f32,
}

struct Enemy {
    enemy_image: graphics::Image,
    position: Vec2,
    velocity: Vec2,
    is_dead: bool
}

struct MainState {
    player_image: graphics::Image,
    player_pos: glam::Vec2,
    player_rotation: f32,

    bullets: Vec<Bullet>,
    enemies: Vec<Enemy>,
    shoot_cooldown: f32,

    shoot_sound: audio::Source,
    pop: audio::Source,
    die: audio::Source,
    next: audio::Source,
    score: i32,
    player_dead: bool
}

impl MainState {
    fn new(ctx: &mut Context) -> Self {
        let (width, height) = graphics::drawable_size(ctx);

        let shoot_sound = audio::Source::new(ctx, "/shoot.wav").unwrap();
        let pop = audio::Source::new(ctx, "/pop.wav").unwrap();
        let die = audio::Source::new(ctx, "/die.wav").unwrap();
        let next = audio::Source::new(ctx, "/new.wav").unwrap();

        MainState {
           player_image: graphics::Image::new(ctx, "/player.png").unwrap(),
           player_pos: glam::Vec2::new(width*0.5,height*0.5),
           player_rotation: 0.0,
           bullets: Vec::new(),
           enemies: Vec::new(),
           shoot_cooldown: 0.0,
           shoot_sound,
           pop,
           die,
           next,
           score: 0,
           player_dead: false
        }
    }

    fn remove_useless_things(&mut self) {
        self.bullets.retain(|x| x.lifetime > 0.0);
        self.enemies.retain(|x| x.is_dead == false);
    }

    fn clear_entities(&mut self) {
        self.bullets.clear();
        self.enemies.clear();
    }
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        const FPS: u32 = 60;

        while ggez::timer::check_update_time(ctx, FPS) {
            if self.player_dead { 
                self.score = 0;
                self.clear_entities();
                self.die.play(ctx).unwrap();
            }

            self.shoot_cooldown -= 0.01;

            let dt = ggez::timer::delta(ctx).as_secs_f32();
            let (scr_width, scr_height) = graphics::drawable_size(ctx);
        
            for bullet in &mut self.bullets { 
                // Changing position of existing bullets
                bullet.position += bullet.velocity * dt;

                // Check if bullet is going off-screen
               if bullet.position.y < 8.0 {
                   bullet.velocity.y = bullet.velocity.y.abs()
               } else if bullet.position.y > scr_height - 8.0 { 
                   bullet.velocity.y = -bullet.velocity.y.abs()
               } else if bullet.position.x < 8.0 {
                    bullet.velocity.x = bullet.velocity.x.abs()
               } else if bullet.position.x > scr_width - 8.0 { 
                    bullet.velocity.x = -bullet.velocity.x.abs()
               }

               bullet.lifetime -= 0.1;
            }
            
            for enemy in &mut self.enemies {
                enemy.position += enemy.velocity * dt;

                if enemy.position.y < 8.0 {
                    enemy.velocity.y = enemy.velocity.y.abs()
                } else if enemy.position.y > scr_height - 8.0 { 
                    enemy.velocity.y = -enemy.velocity.y.abs()
                } else if enemy.position.x < 8.0 {
                    enemy.velocity.x = enemy.velocity.x.abs()
                } else if enemy.position.x > scr_width - 8.0 { 
                    enemy.velocity.x = -enemy.velocity.x.abs()
                }

                // Check bullet collisions with enemies
                for bullet in &mut self.bullets {
                    if  bullet.position.x < enemy.position.x + 32.0 &&
                        bullet.position.x + 16.0 > enemy.position.x &&
                        bullet.position.y < enemy.position.y + 32.0 &&
                        16.0 + bullet.position.y > enemy.position.y {
                        
                        enemy.is_dead = true;
                        bullet.lifetime = 0.0;
                        self.pop.play(ctx).unwrap();
                        self.score += 5;
                    }
                }

                // If enemy collide with player
                if  self.player_pos.x < enemy.position.x + 32.0 &&
                    self.player_pos.x + 32.0 > enemy.position.x &&
                    self.player_pos.y < enemy.position.y + 32.0 &&
                    32.0 + self.player_pos.y > enemy.position.y {
                
                    self.player_dead = true;
                }
            }
            // Rotating the player with keys
            // if keyboard::is_key_pressed(ctx, KeyCode::D) {
            //     self.player_rotation +=  dt * 2.0;
            // }else if keyboard::is_key_pressed(ctx, KeyCode::A) {
            //     self.player_rotation -=  dt * 2.0;
            // }
    
            // Player rotating towards mouse pointer
            let mouse_cords = mouse::position(ctx);
            let dx = self.player_pos.x - mouse_cords.x;
            let dy = self.player_pos.y - mouse_cords.y;
            self.player_rotation = dy.atan2(dx) - 90.0;

            // Shooting
            if mouse::button_pressed(ctx, MouseButton::Left) && self.shoot_cooldown <= 0.0 {
                self.shoot_cooldown = 0.5;
                
                // Make a bullet, set it's rotation and position to the player's rotation and position 
                let bullet_direction = vec_from_angle(-self.player_rotation);

                let bullet = Bullet {
                    bullet_image: graphics::Image::new(ctx, "/shot.png").unwrap(),
                    position: self.player_pos,
                    velocity: -(200.0 * bullet_direction),
                    lifetime: 20.0,

                };
                
                // Slap it into the bullets vector
                self.bullets.push(bullet);
                self.shoot_sound.play(ctx).unwrap();
            } 
            
            // Spawning group of enemies if no enemies left
            if self.enemies.len() == 0 {
                self.clear_entities();

                if self.player_dead { 
                    self.player_dead = false;
                } else {
                    self.next.play(ctx).unwrap();
                }

                let enemy_count: i32 = rand::thread_rng().gen_range(4..8);
                for _ in 0..enemy_count {
                    let rand_x = generate_random_scoords(0.0, scr_width);
                    let rand_y = generate_random_scoords(0.0, scr_height);
                    let enemy = Enemy {
                        enemy_image: graphics::Image::new(ctx,"/enemy.png").unwrap(),
                        position: Vec2::new(rand_x, rand_y),
                        velocity: Vec2::new(rand::thread_rng().gen_range(-30.0..70.0), rand::thread_rng().gen_range(-30.0..50.0)),
                        is_dead: false
                    };

                    self.enemies.push(enemy);
                }
            }

            self.remove_useless_things();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color{r:0.19608 , b:0.19608 , g:0.19608, a:1.0});
        
        // Draw Player
        let player_drawparams = DrawParam::new().dest(self.player_pos).rotation(self.player_rotation).offset(glam::Vec2::new(0.5,0.5));
        graphics::draw(ctx, &self.player_image, player_drawparams).unwrap();

        // Draw Bullets
        for bullet in &self.bullets {
            let bullet_params = DrawParam::new().dest(bullet.position).offset(Vec2::new(0.5,0.5)).scale(Vec2::new(bullet.lifetime / 16.0,bullet.lifetime / 16.0));
            graphics::draw(ctx, &bullet.bullet_image, bullet_params).unwrap();
        }

        // Draw Enemies
        for enemy in &self.enemies {
            let enemyparams = DrawParam::new().dest(enemy.position).offset(Vec2::new(0.5,0.5));
            graphics::draw(ctx, &enemy.enemy_image, enemyparams).unwrap();
        }

        // Draw Score Text
        let guiparams = DrawParam::new().dest(Vec2::new(0.0,0.0));
        let mut text = Text::new(format!("Score: {}", self.score.to_string()));
        text.set_font(Font::new(ctx, "/font.ttf").unwrap(), PxScale::from(24.0));
        graphics::draw(ctx, &text, guiparams).unwrap();

        graphics::present(ctx).unwrap();
        Ok(())
    }
}


fn main() {
    // Loading the resource folder into the cargo directory
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };
    
    // Setting up the window and running
    let (mut ctx, event_loop) = ContextBuilder::new("Shoot balls at balls", "RefinedDev")
    .window_setup(ggez::conf::WindowSetup::default().title("Shoot balls at balls").vsync(true))
    .add_resource_path(resource_dir)
        .build()
        .expect("aieee, could not create ggez context!");
        
    let the_game = MainState::new(&mut ctx);

    ggez::event::run(ctx, event_loop, the_game);
}
