use macroquad::prelude::*;

fn conf() -> Conf {
    Conf {
        window_title: String::from("JUNGLEBEAST"),
        window_width: 1260,
        window_height: 768,
        fullscreen: false,
        ..Default::default()
    }
}

struct Game;

impl Game {
    async fn main(&self) {
        let mut x = 0.0;
        let mut switch = false;
        let bounds = 8.0;

        let world_up = vec3(0.0, 1.0, 0.0);
        let mut yaw: f32 = 1.18;
        let mut pitch: f32 = 0.0;

        let mut front = vec3(
            yaw.cos() * pitch.cos(),
            pitch.sin(),
            yaw.sin() * pitch.cos(),
        )
        .normalize();
        let mut right = front.cross(world_up).normalize();

        let mut position = vec3(0.0, 1.0, 0.0);
        let mut last_mouse_position: Vec2 = mouse_position().into();

        let move_speed = 0.1;
        let look_speed = 0.1;

        set_cursor_grab(true);
        show_mouse(false);

        loop {
            let delta = get_frame_time();

            let step_ws = vec3(right.z, right.y, -right.x) * move_speed;
            let step_ad = right * move_speed;

            if is_key_down(KeyCode::W) { position += step_ws; }
            if is_key_down(KeyCode::S) { position -= step_ws; }
            if is_key_down(KeyCode::A) { position -= step_ad; }
            if is_key_down(KeyCode::D) { position += step_ad; }

            let mouse_position: Vec2 = mouse_position().into();
            let mouse_delta = mouse_position - last_mouse_position;

            last_mouse_position = mouse_position;

            yaw += mouse_delta.x * delta * look_speed;
            pitch += mouse_delta.y * delta * -look_speed;

            pitch = if pitch > 1.5 { 1.5 } else { pitch };
            pitch = if pitch < -1.5 { -1.5 } else { pitch };

            front = vec3(
                yaw.cos() * pitch.cos(),
                pitch.sin(),
                yaw.sin() * pitch.cos(),
            )
            .normalize();

            right = front.cross(world_up).normalize();
            let up = right.cross(front).normalize();

            x += if switch { 0.04 } else { -0.04 };
            if x >= bounds || x <= -bounds {
                switch = !switch;
            }

            clear_background(LIGHTGRAY);

            set_camera(&Camera3D {
                position,
                up,
                target: position + front,
                fovy: 90.0,
                ..Default::default()
            });

            draw_grid(20, 1., BLACK, GRAY);

            set_default_camera();

            let center = (screen_width()/2.0, screen_height()/2.0);
            let crosshair_size = 12.0;
            draw_line(center.0 - crosshair_size, center.1, center.0 + crosshair_size, center.1, 1.0, BLACK);
            draw_line(center.0, center.1 - crosshair_size, center.0, center.1 + crosshair_size, 1.0, BLACK);

            draw_text("JUNGLEBEAST", 10.0, 30.0, 30.0, RED);

            next_frame().await
        }
    }
}

#[macroquad::main(conf)]
async fn main() {
    let game = Game {};
    game.main().await;
}
