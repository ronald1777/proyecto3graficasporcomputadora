use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::f32::consts::PI;
use std::time::Instant;

#[derive(Clone, Copy, Default)]
struct V3 {
    x: f32,
    y: f32,
    z: f32,
}
impl V3 {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    fn dot(self, b: Self) -> f32 {
        self.x * b.x + self.y * b.y + self.z * b.z
    }
    fn norm(self) -> Self {
        let l = self.dot(self).sqrt().max(1e-6);
        self * (1. / l)
    }
    fn cross(self, b: Self) -> Self {
        Self::new(
            self.y * b.z - self.z * b.y,
            self.z * b.x - self.x * b.z,
            self.x * b.y - self.y * b.x,
        )
    }
}
use std::ops::{Add, Mul, Sub};
impl Add for V3 {
    type Output = Self;
    fn add(self, b: Self) -> Self {
        Self::new(self.x + b.x, self.y + b.y, self.z + b.z)
    }
}
impl Sub for V3 {
    type Output = Self;
    fn sub(self, b: Self) -> Self {
        Self::new(self.x - b.x, self.y - b.y, self.z - b.z)
    }
}
impl Mul<f32> for V3 {
    type Output = Self;
    fn mul(self, s: f32) -> Self {
        Self::new(self.x * s, self.y * s, self.z * s)
    }
}
#[derive(Clone, Copy)]
struct Tri {
    a: V3,
    b: V3,
    c: V3,
    color: u32,
    textured: bool,
}
fn parse_obj(path: &str) -> Vec<Tri> {
    let Ok(s) = std::fs::read_to_string(path) else {
        return vec![];
    };
    let mut vs = vec![];
    let mut out = vec![];
    for l in s.lines() {
        let p: Vec<_> = l.split_whitespace().collect();
        if p.first() == Some(&"v") {
            if p.len() >= 4 {
                vs.push(V3::new(
                    p[1].parse().unwrap_or(0.),
                    p[2].parse().unwrap_or(0.),
                    p[3].parse().unwrap_or(0.),
                ));
            }
        } else if p.first() == Some(&"f") {
            let ids: Vec<usize> = p[1..]
                .iter()
                .filter_map(|v| v.split('/').next()?.parse::<usize>().ok())
                .map(|n| n - 1)
                .collect();
            for i in 1..ids.len().saturating_sub(1) {
                if ids[0] < vs.len() && ids[i] < vs.len() && ids[i + 1] < vs.len() {
                    let a = vs[ids[0]];
                    let b = vs[ids[i]];
                    let c = vs[ids[i + 1]];
                    out.push(Tri {
                        a,
                        b,
                        c,
                        color: ship_color(a, b, c),
                        textured: false,
                    })
                }
            }
        }
    }
    out
}
fn ship_color(a: V3, b: V3, c: V3) -> u32 {
    let center = (a + b + c) * (1.0 / 3.0);
    if center.y > 3.05 {
        0x69777a // cockpit canopy / shaded metal
    } else if center.y < 1.8 {
        0x554d43 // dusty dark underside
    } else if center.x.abs() > 1.45 && center.y > 2.35 {
        0x9f4e39 // muted rust-red wing markings
    } else if center.z < -2.15 && center.x.abs() > 0.65 {
        0x414a4c // engine housings
    } else {
        0xc6ad83 // sand-worn Tatooine hull
    }
}
fn view_transform(v: V3, yaw: f32, pitch: f32, roll: f32) -> V3 {
    // Inversa de la orientación de la cámara: pitch, yaw y alabeo.
    let (sp, cp) = pitch.sin_cos();
    let (sy, cy) = yaw.sin_cos();
    let y1 = v.y * cp - v.z * sp;
    let z1 = v.y * sp + v.z * cp;
    let x2 = v.x * cy + z1 * sy;
    let z2 = -v.x * sy + z1 * cy;
    let (sr, cr) = roll.sin_cos();
    V3::new(x2 * cr + y1 * sr, -x2 * sr + y1 * cr, z2)
}
fn orient_ship(v: V3, yaw: f32, pitch: f32, roll: f32) -> V3 {
    let (sr, cr) = roll.sin_cos();
    rotate(
        V3::new(v.x * cr - v.y * sr, v.x * sr + v.y * cr, v.z),
        yaw,
        pitch,
    )
}
fn make_stars(count: usize) -> Vec<(V3, u32, bool)> {
    let mut state = 0x6d2b_79f5u32;
    let mut random = || {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        state as f32 / u32::MAX as f32
    };
    (0..count)
        .map(|i| {
            let mut direction;
            loop {
                direction = V3::new(
                    random() * 2.0 - 1.0,
                    random() * 2.0 - 1.0,
                    random() * 2.0 - 1.0,
                );
                if direction.dot(direction) > 0.03 && direction.dot(direction) < 1.0 {
                    break;
                }
            }
            let color = match i % 9 {
                0 => 0xe7d8b2,
                1 => 0x9dbbe8,
                _ => 0x9aa9c7,
            };
            (direction.norm(), color, i % 23 == 0)
        })
        .collect()
}
fn sphere(lat: usize, lon: usize, color: u32, texture: bool) -> Vec<Tri> {
    let mut o = vec![];
    for y in 0..lat {
        let a = PI * y as f32 / lat as f32;
        let b = PI * (y + 1) as f32 / lat as f32;
        for x in 0..lon {
            let c = 2. * PI * x as f32 / lon as f32;
            let d = 2. * PI * (x + 1) as f32 / lon as f32;
            let p = |u: f32, v: f32| V3::new(u.sin() * v.cos(), u.cos(), u.sin() * v.sin());
            let (p0, p1, p2, p3) = (p(a, c), p(a, d), p(b, c), p(b, d));
            o.push(Tri {
                a: p0,
                b: p2,
                c: p1,
                color,
                textured: texture,
            });
            o.push(Tri {
                a: p1,
                b: p2,
                c: p3,
                color,
                textured: texture,
            });
        }
    }
    o
}
struct Cam {
    p: V3,
    yaw: f32,
    pitch: f32,
    roll: f32,
}
fn rotate(v: V3, y: f32, p: f32) -> V3 {
    let (sy, cy) = y.sin_cos();
    let (sp, cp) = p.sin_cos();
    let x = v.x * cy - v.z * sy;
    let z = v.x * sy + v.z * cy;
    V3::new(x, z * sp + v.y * cp, z * cp - v.y * sp)
}
// (radio orbital, fase inicial, radio del planeta, textura, tinte, velocidad angular)
const PLANETS: [(f32, f32, f32, usize, u32, f32); 12] = [
    (5.0, 0.0, 1.00, 1, 0xffffff, 0.20),
    (8.5, 2.0, 1.30, 2, 0xa9dfff, 0.16),
    (12.0, 4.0, 0.78, 1, 0xffbd8a, 0.13),
    (16.0, 1.1, 0.95, 2, 0xa7e3ce, 0.105),
    (20.5, 3.2, 1.15, 1, 0xf0d792, 0.090),
    (25.5, 5.1, 0.85, 2, 0xc7b2f2, 0.078),
    (31.0, 0.4, 1.40, 1, 0xf2a8a8, 0.068),
    (37.0, 2.5, 0.82, 2, 0x9fbcef, 0.059),
    (44.0, 4.7, 1.10, 1, 0xc8e5a2, 0.052),
    (52.0, 1.7, 0.90, 2, 0xf2c39c, 0.046),
    (61.0, 3.8, 1.25, 1, 0xb0d6d8, 0.041),
    (72.0, 5.6, 1.05, 2, 0xd6b7ec, 0.036),
];
fn tint_color(color: u32, tint: u32) -> u32 {
    let r = ((color >> 16) & 255) * ((tint >> 16) & 255) / 255;
    let g = ((color >> 8) & 255) * ((tint >> 8) & 255) / 255;
    let b = (color & 255) * (tint & 255) / 255;
    (r << 16) | (g << 8) | b
}

fn main() {
    let mut window = Window::new(
        "ORBITA | renderer de software",
        960,
        600,
        WindowOptions {
            resize: true,
            ..WindowOptions::default()
        },
    )
    .expect("No se pudo crear ventana");
    window.set_target_fps(60);
    let mut fast = true;
    let mut cam = Cam {
        p: V3::new(0., 0., -18.),
        yaw: 0.,
        pitch: 0.,
        roll: 0.,
    };
    let (mut w, mut h) = (480usize, 300usize);
    let mut buf = vec![0u32; w * h];
    let mut present = vec![0u32; 960 * 600];
    let start = Instant::now();
    let mut frame_clock = Instant::now();
    let mut speed = 0.0f32;
    let mut lateral_speed = 0.0f32;
    let mut vertical_speed = 0.0f32;
    let mut pitch_rate = 0.0f32;
    let mut camera_eye = V3::new(0.0, 1.8, -24.0);
    let mut mouse_prev = false;
    let ship = parse_obj("assets/nave_ronald.obj");
    let sphere_fast = sphere(12, 20, 0xffffff, true);
    let sphere_normal = sphere(28, 44, 0xffffff, true);
    let stars = make_stars(650);
    let texs = [
        "assets/textures/star.png",
        "assets/textures/rock.png",
        "assets/textures/gas.png",
    ];
    let textures: Vec<Vec<u32>> = texs
        .iter()
        .map(|p| {
            image::open(p)
                .map(|i| {
                    i.to_rgb8()
                        .pixels()
                        .map(|c| ((c[0] as u32) << 16) | ((c[1] as u32) << 8) | c[2] as u32)
                        .collect()
                })
                .unwrap_or_default()
        })
        .collect();
    let mut depth = vec![f32::INFINITY; w * h];
    let mut scene: Vec<(&Vec<Tri>, V3, f32, usize, f32, u32)> = Vec::with_capacity(14);
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = start.elapsed().as_secs_f32();
        let dt = frame_clock.elapsed().as_secs_f32().clamp(0.001, 0.05);
        frame_clock = Instant::now();
        let mouse = window
            .get_mouse_pos(minifb::MouseMode::Discard)
            .unwrap_or((-1.0, -1.0));
        let clicked = window.get_mouse_down(minifb::MouseButton::Left)
            && !mouse_prev
            && mouse.0 >= 8.0
            && mouse.0 <= 260.0
            && mouse.1 >= 8.0
            && mouse.1 <= 48.0;
        mouse_prev = window.get_mouse_down(minifb::MouseButton::Left);
        if window.is_key_pressed(Key::Tab, KeyRepeat::No) || clicked {
            fast = !fast;
            (w, h) = if fast { (480, 300) } else { (800, 500) };
            buf = vec![0; w * h];
            depth = vec![f32::INFINITY; w * h];
        }
        // Vuelo arcade: aceleración/freno progresivos y control en tres ejes.
        if window.is_key_down(Key::R) {
            speed = (speed + dt * 5.5).min(9.0);
        } else if window.is_key_down(Key::F) {
            speed = (speed - dt * 8.0).max(-2.5);
        } else {
            speed *= (-0.9 * dt).exp();
            if speed.abs() < 0.025 {
                speed = 0.0;
            }
        }
        let forward = rotate(V3::new(0.0, 0.0, 1.0), cam.yaw, cam.pitch);
        cam.p = cam.p + forward * (speed * dt);
        let right = rotate(V3::new(1.0, 0.0, 0.0), cam.yaw, 0.0);
        let side_input = (if window.is_key_down(Key::G) { 1.0 } else { 0.0 })
            - (if window.is_key_down(Key::D) { 1.0 } else { 0.0 });
        let side_target = side_input * 4.0;
        lateral_speed += (side_target - lateral_speed) * (1.0 - (-3.2 * dt).exp());
        cam.p = cam.p + right * (lateral_speed * dt);

        let bank_target = if window.is_key_down(Key::Left) {
            0.58
        } else if window.is_key_down(Key::Right) {
            -0.58
        } else {
            0.0
        };
        // Inclinación, giro y nivelación progresivos.
        cam.roll += (bank_target - cam.roll) * (1.0 - (-2.8 * dt).exp());
        cam.yaw += cam.roll * dt * (0.25 + speed.abs() * 0.045);

        let pitch_input = (if window.is_key_down(Key::Up) {
            1.0
        } else {
            0.0
        }) - (if window.is_key_down(Key::Down) {
            1.0
        } else {
            0.0
        });
        let pitch_target = pitch_input * 0.7;
        pitch_rate += (pitch_target - pitch_rate) * (1.0 - (-3.5 * dt).exp());
        cam.pitch = (cam.pitch + pitch_rate * dt).clamp(-1.2, 1.2);

        let vertical_input = (if window.is_key_down(Key::T) { 1.0 } else { 0.0 })
            - (if window.is_key_down(Key::C) { 1.0 } else { 0.0 });
        let vertical_target = vertical_input * 4.0;
        vertical_speed += (vertical_target - vertical_speed) * (1.0 - (-3.2 * dt).exp());
        cam.p.y += vertical_speed * dt;
        for &(rad, phase, size, _, _, orbit_speed) in &PLANETS {
            let angle = now * orbit_speed + phase;
            let center = V3::new(angle.cos() * rad, 0.0, angle.sin() * rad);
            let delta = cam.p - center;
            let dist = delta.dot(delta).sqrt();
            let safe = size + 0.65;
            if dist < safe {
                cam.p = center + delta.norm() * safe;
            }
        }
        let heading = rotate(V3::new(0.0, 0.0, 1.0), cam.yaw, cam.pitch);
        let desired_eye = cam.p - heading * 6.0 + V3::new(0.0, 1.8, 0.0);
        camera_eye = camera_eye + (desired_eye - camera_eye) * (1.0 - (-3.5 * dt).exp());
        let camera_pitch = (cam.pitch - 0.29).clamp(-1.2, 1.35);
        let camera_roll = cam.roll * 0.55;
        buf.fill(0x030611);
        depth.fill(f32::INFINITY);
        // Skybox procedural: direcciones fijas que solo cambian con la orientación.
        let star_focal = w as f32 * 0.58;
        for (direction, color, bright) in &stars {
            let q = view_transform(*direction, cam.yaw, camera_pitch, camera_roll);
            if q.z <= 0.05 {
                continue;
            }
            let sx = (w as f32 * 0.5 + q.x / q.z * star_focal) as i32;
            let sy = (h as f32 * 0.5 - q.y / q.z * star_focal) as i32;
            if sx >= 0 && sy >= 0 && (sx as usize) < w && (sy as usize) < h {
                buf[sy as usize * w + sx as usize] = *color;
                if *bright && sx + 1 < w as i32 && sy + 1 < h as i32 {
                    buf[(sy as usize + 1) * w + sx as usize] = *color;
                    buf[sy as usize * w + sx as usize + 1] = *color;
                }
            }
        }
        let mut tris = 0usize;
        scene.clear();
        let sphere_mesh = if fast { &sphere_fast } else { &sphere_normal };
        scene.push((sphere_mesh, V3::new(0.0, 0.0, 0.0), 1.2, 0, now, 0xffffff));
        for &(rad, phase, size, texture, tint, orbit_speed) in &PLANETS {
            let angle = now * orbit_speed + phase;
            let position = V3::new(angle.cos() * rad, 0.0, angle.sin() * rad);
            scene.push((sphere_mesh, position, size, texture, now, tint));
        }
        // orbit rings are software triangles, rendered as thin sampled line segments via depth-tested pixel writes
        let orbit_steps = if fast { 240 } else { 480 };
        for &(rad, _, _, _, _, _) in &PLANETS {
            let mut previous: Option<(i32, i32)> = None;
            for n in 0..orbit_steps {
                let a = n as f32 * 2.0 * PI / orbit_steps as f32;
                let pos = V3::new(a.cos() * rad, 0.0, a.sin() * rad);
                let q = view_transform(pos - camera_eye, cam.yaw, camera_pitch, camera_roll);
                if q.z > 0.6 {
                    let sx = (w as f32 * 0.5 + q.x / q.z * w as f32 * 0.55) as i32;
                    let sy = (h as f32 * 0.5 - q.y / q.z * w as f32 * 0.55) as i32;
                    let on_screen = sx >= 0 && sy >= 0 && (sx as usize) < w && (sy as usize) < h;
                    if on_screen {
                        if let Some((px, py)) = previous {
                            // Projection can jump sharply where a distant orbit approaches
                            // the horizon. Start a new visible section instead of drawing
                            // a long streak across the screen.
                            let jump_limit = 24;
                            if (sx - px).abs() <= jump_limit && (sy - py).abs() <= jump_limit {
                                draw_line(&mut buf, w, h, px, py, sx, sy, 0x6685b5);
                            }
                        }
                        previous = Some((sx, sy));
                    } else {
                        previous = None;
                    }
                } else {
                    previous = None;
                }
            }
        }
        // world objects and CPU triangle rasterization with depth test and fragment shader
        scene.push((&ship, cam.p, 0.20, usize::MAX, now, 0xffffff));
        for &(object, pos, scale, tx, time, tint) in &scene {
            let tex = if tx < textures.len() {
                &textures[tx]
            } else {
                &textures[0]
            };
            for t in object.iter() {
                tris += 1;
                let world = |v: V3| {
                    if tx == usize::MAX {
                        let centered = v - V3::new(-0.16, 2.5, 0.43);
                        pos + orient_ship(centered * scale, cam.yaw, cam.pitch, cam.roll)
                    } else {
                        let spin = time * (0.15 + tx as f32 * 0.07);
                        let (sn, cs) = spin.sin_cos();
                        pos + V3::new(
                            (v.x * cs - v.z * sn) * scale,
                            v.y * scale,
                            (v.x * sn + v.z * cs) * scale,
                        )
                    }
                };
                let wa = world(t.a);
                let wb = world(t.b);
                let wc = world(t.c);
                let ca = view_transform(wa - camera_eye, cam.yaw, camera_pitch, camera_roll);
                let cb = view_transform(wb - camera_eye, cam.yaw, camera_pitch, camera_roll);
                let cc = view_transform(wc - camera_eye, cam.yaw, camera_pitch, camera_roll);
                if ca.z <= 0.12 || cb.z <= 0.12 || cc.z <= 0.12 {
                    continue;
                }
                let focal = w as f32 * 0.58;
                let project = |v: V3| {
                    (
                        w as f32 * 0.5 + v.x / v.z * focal,
                        h as f32 * 0.5 - v.y / v.z * focal,
                        v.z,
                    )
                };
                let (ax, ay, az) = project(ca);
                let (bx, by, bz) = project(cb);
                let (cx, cy, cz) = project(cc);
                let minx = ax.min(bx).min(cx).floor().max(0.) as usize;
                let maxx = ax.max(bx).max(cx).ceil().min(w as f32 - 1.) as usize;
                let miny = ay.min(by).min(cy).floor().max(0.) as usize;
                let maxy = ay.max(by).max(cy).ceil().min(h as f32 - 1.) as usize;
                if maxx < minx || maxy < miny {
                    continue;
                }
                let area = (bx - ax) * (cy - ay) - (by - ay) * (cx - ax);
                if area.abs() < 0.01 {
                    continue;
                }
                let normal = (wb - wa).cross(wc - wa).norm();
                let light =
                    (0.25 + normal.dot(V3::new(-0.4, 0.7, -0.5).norm()).max(0.) * 0.75).min(1.);
                for y in miny..=maxy {
                    for x in minx..=maxx {
                        let px = x as f32 + 0.5;
                        let py = y as f32 + 0.5;
                        let u = ((bx - px) * (cy - py) - (by - py) * (cx - px)) / area;
                        let v = ((cx - px) * (ay - py) - (cy - py) * (ax - px)) / area;
                        let z = u * az + v * bz + (1. - u - v) * cz;
                        let idx = y * w + x;
                        if u >= 0. && v >= 0. && u + v <= 1. && z < depth[idx] {
                            depth[idx] = z;
                            let base = if t.textured && !tex.is_empty() {
                                let wc = 1.0 - u - v;
                                let iz = u / az + v / bz + wc / cz;
                                let local = (t.a * (u / az / iz)
                                    + t.b * (v / bz / iz)
                                    + t.c * (wc / cz / iz))
                                    .norm();
                                // Estos PNG son renders ortográficos de una esfera sobre negro,
                                // no mapas UV. Muestreamos el disco de la imagen según la normal.
                                let tx = (180.0 + local.x * 134.0).clamp(0.0, 359.0) as usize;
                                let ty = (180.0 - local.y * 134.0).clamp(0.0, 359.0) as usize;
                                tex[(ty * 360 + tx).min(tex.len() - 1)]
                            } else {
                                t.color
                            };
                            let base = tint_color(base, tint);
                            let r = (((base >> 16) & 255) as f32 * light) as u32;
                            let g = (((base >> 8) & 255) as f32 * light) as u32;
                            let b = ((base & 255) as f32 * light) as u32;
                            buf[idx] = (r << 16) | (g << 8) | b;
                        }
                    }
                }
            }
        }
        // HUD: crisp software-drawn panels and quality button
        let button = if fast {
            "MODO OPTIMIZADO  [TAB]  |  480x300  |  12 planetas"
        } else {
            "MODO NORMAL  [TAB]  |  800x500  |  12 planetas"
        };
        draw_rect(&mut buf, w, h, 8, 8, 260, 28, 0x15243b);
        draw_text(
            &mut buf,
            w,
            h,
            14,
            14,
            "ORBITA  /  SISTEMA HELIOS",
            0xd6e9ff,
        );
        draw_text(&mut buf, w, h, 14, 34, button, 0x65e6cc);
        draw_text(
            &mut buf,
            w,
            h,
            14,
            h.saturating_sub(27),
            "R ACELERA F FRENA FLECHAS GIRAN D/G LADO T/C ALTURA",
            0xb8c5da,
        );
        draw_text(
            &mut buf,
            w,
            h,
            w.saturating_sub(68),
            h.saturating_sub(27),
            &format!("TRIS {}", tris),
            0xffcf8d,
        );
        if window.is_key_pressed(Key::Tab, KeyRepeat::No) {
            fast = !fast;
            (w, h) = if fast { (480, 300) } else { (800, 500) };
            buf = vec![0; w * h];
            depth = vec![f32::INFINITY; w * h];
        }
        // Escalado nearest-neighbour del framebuffer CPU al tamaño de la ventana.
        let (out_w, out_h) = window.get_size();
        present.resize(out_w * out_h, 0);
        for oy in 0..out_h {
            let sy = oy * h / out_h;
            for ox in 0..out_w {
                let sx = ox * w / out_w;
                present[oy * out_w + ox] = buf[sy * w + sx];
            }
        }
        window.update_with_buffer(&present, out_w, out_h).unwrap();
    }
}
fn draw_text(b: &mut [u32], w: usize, h: usize, x: usize, y: usize, s: &str, c: u32) {
    for (i, ch) in s.chars().enumerate() {
        let rows = glyph_rows(ch);
        for (yy, row) in rows.iter().enumerate() {
            for (xx, pixel) in row.bytes().enumerate() {
                if pixel == b'1' {
                    let (px, py) = (x + i * 6 + xx, y + yy);
                    if px < w && py < h {
                        b[py * w + px] = c;
                    }
                }
            }
        }
    }
}
fn glyph_rows(ch: char) -> [&'static str; 7] {
    match ch.to_ascii_uppercase() {
        ' ' => [
            "00000", "00000", "00000", "00000", "00000", "00000", "00000",
        ],
        'A' => [
            "01110", "10001", "10001", "11111", "10001", "10001", "10001",
        ],
        'B' => [
            "11110", "10001", "10001", "11110", "10001", "10001", "11110",
        ],
        'C' => [
            "01111", "10000", "10000", "10000", "10000", "10000", "01111",
        ],
        'D' => [
            "11110", "10001", "10001", "10001", "10001", "10001", "11110",
        ],
        'E' => [
            "11111", "10000", "10000", "11110", "10000", "10000", "11111",
        ],
        'F' => [
            "11111", "10000", "10000", "11110", "10000", "10000", "10000",
        ],
        'G' => [
            "01111", "10000", "10000", "10111", "10001", "10001", "01111",
        ],
        'H' => [
            "10001", "10001", "10001", "11111", "10001", "10001", "10001",
        ],
        'I' => [
            "11111", "00100", "00100", "00100", "00100", "00100", "11111",
        ],
        'J' => [
            "00111", "00010", "00010", "00010", "10010", "10010", "01100",
        ],
        'K' => [
            "10001", "10010", "10100", "11000", "10100", "10010", "10001",
        ],
        'L' => [
            "10000", "10000", "10000", "10000", "10000", "10000", "11111",
        ],
        'M' => [
            "10001", "11011", "10101", "10101", "10001", "10001", "10001",
        ],
        'N' => [
            "10001", "11001", "10101", "10011", "10001", "10001", "10001",
        ],
        'O' => [
            "01110", "10001", "10001", "10001", "10001", "10001", "01110",
        ],
        'P' => [
            "11110", "10001", "10001", "11110", "10000", "10000", "10000",
        ],
        'Q' => [
            "01110", "10001", "10001", "10001", "10101", "10010", "01101",
        ],
        'R' => [
            "11110", "10001", "10001", "11110", "10100", "10010", "10001",
        ],
        'S' => [
            "01111", "10000", "10000", "01110", "00001", "00001", "11110",
        ],
        'T' => [
            "11111", "00100", "00100", "00100", "00100", "00100", "00100",
        ],
        'U' => [
            "10001", "10001", "10001", "10001", "10001", "10001", "01110",
        ],
        'V' => [
            "10001", "10001", "10001", "10001", "10001", "01010", "00100",
        ],
        'W' => [
            "10001", "10001", "10001", "10101", "10101", "10101", "01010",
        ],
        'X' => [
            "10001", "10001", "01010", "00100", "01010", "10001", "10001",
        ],
        'Y' => [
            "10001", "10001", "01010", "00100", "00100", "00100", "00100",
        ],
        'Z' => [
            "11111", "00001", "00010", "00100", "01000", "10000", "11111",
        ],
        '0' => [
            "01110", "10001", "10011", "10101", "11001", "10001", "01110",
        ],
        '1' => [
            "00100", "01100", "00100", "00100", "00100", "00100", "01110",
        ],
        '2' => [
            "01110", "10001", "00001", "00010", "00100", "01000", "11111",
        ],
        '3' => [
            "11110", "00001", "00001", "01110", "00001", "00001", "11110",
        ],
        '4' => [
            "00010", "00110", "01010", "10010", "11111", "00010", "00010",
        ],
        '5' => [
            "11111", "10000", "10000", "11110", "00001", "00001", "11110",
        ],
        '6' => [
            "01110", "10000", "10000", "11110", "10001", "10001", "01110",
        ],
        '7' => [
            "11111", "00001", "00010", "00100", "01000", "01000", "01000",
        ],
        '8' => [
            "01110", "10001", "10001", "01110", "10001", "10001", "01110",
        ],
        '9' => [
            "01110", "10001", "10001", "01111", "00001", "00001", "01110",
        ],
        ':' => [
            "00000", "00100", "00100", "00000", "00100", "00100", "00000",
        ],
        '.' => [
            "00000", "00000", "00000", "00000", "00000", "00100", "00100",
        ],
        '/' => [
            "00001", "00010", "00010", "00100", "01000", "01000", "10000",
        ],
        '[' => [
            "01110", "01000", "01000", "01000", "01000", "01000", "01110",
        ],
        ']' => [
            "01110", "00010", "00010", "00010", "00010", "00010", "01110",
        ],
        '-' => [
            "00000", "00000", "00000", "11111", "00000", "00000", "00000",
        ],
        '|' => [
            "00100", "00100", "00100", "00100", "00100", "00100", "00100",
        ],
        _ => [
            "00000", "00000", "00000", "00000", "00000", "00000", "00000",
        ],
    }
}

fn draw_rect(b: &mut [u32], w: usize, h: usize, x: usize, y: usize, rw: usize, rh: usize, c: u32) {
    for yy in y..(y + rh).min(h) {
        for xx in x..(x + rw).min(w) {
            b[yy * w + xx] = c
        }
    }
}

fn draw_line(
    buf: &mut [u32],
    w: usize,
    h: usize,
    mut x0: i32,
    mut y0: i32,
    x1: i32,
    y1: i32,
    color: u32,
) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut error = dx + dy;
    loop {
        if x0 >= 0 && y0 >= 0 && (x0 as usize) < w && (y0 as usize) < h {
            buf[y0 as usize * w + x0 as usize] = color;
        }
        if x0 == x1 && y0 == y1 {
            break;
        }
        let twice = 2 * error;
        if twice >= dy {
            error += dy;
            x0 += sx;
        }
        if twice <= dx {
            error += dx;
            y0 += sy;
        }
    }
}
