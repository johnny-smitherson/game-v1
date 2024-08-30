use bevy::{
    log::info,
    prelude::{Reflect, Vec2},
};
use std::{
    cmp::Ordering,
    f32::{consts::PI, INFINITY},
};

pub const TANK_BULLET_SPEED_PER_POWER: f32 = 0.28;
pub const GRAVITY_SCALE: f32 = 1.0;
pub const GRAVITY_MAGNITUDE: f32 = 9.81;
pub const BULLET_DENSITY: f32 = 100.0;
pub const TANK_DENSITY: f32 = 1000.0;
pub const BULLET_LINEAR_DAMPING: f32 = 0.0;

pub const TRAJECTORY_POINTS: usize = 20;

#[derive(Clone, Debug, Reflect)]
pub struct BulletSolution {
    pub elevation: f32,
    pub flight_time: f32,
    pub speed: f32,
    pub power: f32,
    pub trajectory: Vec<Vec2>,
    _next_iter_point: Vec2,
    _absolute_error: Vec2,
}
#[derive(Clone, Debug, Reflect)]
pub struct BulletSolutions {
    pub chosen_sol: Option<BulletSolution>,
    pub all_sol: Vec<BulletSolution>,
    pub err_sol: Option<BulletSolution>,
}

pub fn compute_ballistic_solution(range: f32, _y_diff: f32, max_speed: f32) -> BulletSolutions {
    let alpha = BULLET_LINEAR_DAMPING;
    let gravity = GRAVITY_MAGNITUDE * GRAVITY_SCALE;
    let points: usize = TRAJECTORY_POINTS;
    let pos = Vec2::new(range, _y_diff);
    if alpha > 0.0 {
        // _compute_with_damping_many_iter(pos, speed, gravity, points, alpha)
        panic!("todo!");
    } else {
        _compute_ballistic_solution_no_damping(pos, max_speed, gravity, points)
    }
}

fn _compute_ballistic_solution_no_damping(
    pos: Vec2,
    max_speed: f32,
    gravity: f32,
    points: usize,
) -> BulletSolutions {
    let (range, y_diff) = (pos.x, pos.y);

    let make_solution = |elevation: f32, speed: f32, points: usize| {
        let speed_x = elevation.cos() * speed;
        let speed_y = elevation.sin() * speed;
        let flight_time = range / speed_x;

        let compute_point = |time: f32| {
            Vec2::new(
                time * speed_x,
                time * speed_y - gravity * time.powi(2) / 2.0,
            )
        };

        let trajectory: Vec<_> = (0..points)
            .map(|i| {
                let time = flight_time * (i as f32) / (points as f32 - 1.0);
                compute_point(time)
            })
            .collect();
        let abs_err = Vec2::new(range, y_diff) - trajectory[trajectory.len() - 1];

        BulletSolution {
            elevation,
            flight_time,
            trajectory,
            speed,
            power: speed / TANK_BULLET_SPEED_PER_POWER,
            _absolute_error: abs_err,
            _next_iter_point: Vec2::new(range, y_diff) + abs_err,
        }
    };
    // https://math.stackexchange.com/questions/3019313/finding-projectile-angle-with-different-elevation-when-velocity-and-range-are-kn
    // \theta = \arctan \left( \frac{v_0^2 \pm \sqrt{v_0^4 - g(gx_f^2+2y_fv_0^2)}}{gx_f} \right)
    let base_angles = |range: f32, speed: f32, y_diff: f32| {
        let v0_sq = speed * speed;
        let par = gravity * range.powi(2) + 2.0 * y_diff * v0_sq;
        let sub_radical = v0_sq * v0_sq - gravity * par;
        if sub_radical < 0.0 {
            (false, 0.0, 0.0)
        } else {
            let radical = (sub_radical).sqrt();
            let _ang2 = ((v0_sq + radical) / (gravity * range)).atan();
            let _ang1 = ((v0_sq - radical) / (gravity * range)).atan();
            (true, _ang1, _ang2)
        }
    };
    // let min_speed = {
    //     // v0_sq * v0_sq - gravity * par = 0
    //     // v0_sq * v0_sq - gravity * (gravity * range.powi(2) + 2.0 * y_diff * v0_sq) = 0
    //     // v0_sq * v0_sq - gravity * gravity * range.powi(2) + gravity * 2.0 * y_diff * v0_sq = 0
    //     // aX^2 + bX + c = 0
    //     // a = 1
    //     // b = gravity * 2.0 * y_diff
    //     // c =  gravity * gravity * range.powi(2)
    //     // X = (-b - sqrt(b^2 - 4ac)) / 2a
    //     let a = 1.0;
    //     let b = gravity * 2.0 * y_diff;
    //     let c = -gravity * gravity * range.powi(2);
    //     let d = b * b - 4.0 * a * c;
    //     if d < 0.0 {
    //         INFINITY
    //     } else {
    //         // pick min > 0
    //         let v1_2 = ((-b - d.sqrt()) / (2.0 * a));
    //         let v2_2 = ((-b + d.sqrt()) / (2.0 * a));
    //         if v1_2 < 0.0 {
    //             if v2_2 < 0.0 {
    //                 INFINITY
    //             } else {
    //                 v2_2.sqrt()
    //             }
    //         } else {
    //             v1_2.sqrt()
    //         }
    //     }
    // };
    let (ok1, _, _) = base_angles(range, max_speed, y_diff);
    if !ok1 {
        return BulletSolutions {
            chosen_sol: None,
            all_sol: vec![],
            err_sol: Some(make_solution(PI / 4.0, max_speed, points)),
        };
    }
    const N_SPEEDS: i32 = 20;

    let mut trajectories: Vec<_> = (1..=N_SPEEDS)
        .map(|i| (max_speed) * i as f32 / N_SPEEDS as f32)
        .map(|speed| (speed, base_angles(range, speed, y_diff)))
        .filter(|s| s.1 .0)
        .flat_map(|(speed, (_, _ang1, _ang2))| {
            let t = (
                make_solution(_ang1, speed, points),
                make_solution(_ang2, speed, points),
            );
            std::iter::once(t.0).chain(std::iter::once(t.1))
        })
        .collect();
    trajectories.sort_by(|a, b| a.flight_time.partial_cmp(&b.flight_time).unwrap());
    BulletSolutions {
        chosen_sol: Some(trajectories[0].clone()),
        all_sol: trajectories,
        err_sol: None,
    }
}

// fn _compute_with_damping_many_iter(
//     pos: Vec2,
//     speed: f32,
//     gravity: f32,
//     points: usize,
//     alpha: f32,
// ) -> BulletSolutions {
//     let mut sol0 = _compute_ballistic_solution_with_damping(pos, speed, gravity, points, alpha);
//     let mut lows: Vec<Option<BulletSolution>> = vec![sol0.low_sol.clone()];
//     let mut highs: Vec<Option<BulletSolution>> = vec![sol0.high_sol.clone()];
//     for _ in 0..5 {
//         if let Some(low) = sol0.low_sol {
//             let next = _compute_ballistic_solution_with_damping(
//                 low._next_iter_point,
//                 speed,
//                 gravity,
//                 points,
//                 alpha,
//             );
//             lows.push(next.low_sol.clone());
//             sol0.low_sol = next.low_sol;
//             sol0.err_sol = sol0.err_sol.or(next.err_sol);
//         }
//         if let Some(high) = sol0.high_sol {
//             let next = _compute_ballistic_solution_with_damping(
//                 high._next_iter_point,
//                 speed,
//                 gravity,
//                 points,
//                 alpha,
//             );
//             highs.push(next.high_sol.clone());
//             sol0.high_sol = next.high_sol;
//             sol0.err_sol = sol0.err_sol.or(next.err_sol);
//         }
//     }
//     let cmp = |a: &Option<BulletSolution>, b: &Option<BulletSolution>| {
//         let va = if let Some(x) = a {
//             x.trajectory.last().unwrap().distance(pos)
//         } else {
//             9999999.9_f32
//         };
//         let vb = if let Some(x) = b {
//             x.trajectory.last().unwrap().distance(pos)
//         } else {
//             9999999.9_f32
//         };
//         if va < vb {
//             Ordering::Less
//         } else if va > vb {
//             Ordering::Greater
//         } else {
//             Ordering::Equal
//         }
//     };
//     highs.sort_by(cmp);
//     lows.sort_by(cmp);
//     sol0.low_sol = lows[0].clone();
//     sol0.high_sol = highs[0].clone();
//     sol0
// }

// fn _compute_ballistic_solution_with_damping(
//     pos: Vec2,
//     speed: f32,
//     gravity: f32,
//     points: usize,
//     alpha: f32,
// ) -> BulletSolutions {
//     let (range, y_diff) = (pos.x, pos.y);
//     let make_solution = |elevation: f32, points: usize| {
//         let speed_x = elevation.cos() * speed;
//         let speed_y = elevation.sin() * speed;
//         let mut flight_time = range / speed_x;
//         flight_time = flight_time / ((1.0 - (-alpha * flight_time).exp()) / alpha / flight_time);
//         let compute_point = |time: f32| {
//             // https://www.lehman.edu/faculty/dgaranin/Mathematical_Physics/Mathematical_physics-10-Differential_equations.pdf
//             // page 15
//             let t_exp = (1.0 - (-alpha * time).exp()) / alpha;
//             Vec2::new(
//                 speed_x * t_exp,
//                 (speed_y + gravity / alpha) * t_exp - gravity * time / alpha,
//             )
//         };
//         flight_time *= range / compute_point(flight_time).x;
//         flight_time *= range / compute_point(flight_time).x;
//         flight_time *= range / compute_point(flight_time).x;
//         let trajectory: Vec<_> = (0..points)
//             .map(|i| {
//                 let time = flight_time * (i as f32) / (points as f32 - 1.0);
//                 compute_point(time)
//             })
//             .collect();
//         let abs_err = Vec2::new(range, y_diff) - trajectory[trajectory.len() - 1];

//         BulletSolution {
//             elevation,
//             flight_time,
//             trajectory,
//             _absolute_error: abs_err,
//             _next_iter_point: Vec2::new(range, y_diff) + abs_err * 0.7,
//         }
//     };
//     // https://math.stackexchange.com/questions/3019313/finding-projectile-angle-with-different-elevation-when-velocity-and-range-are-kn
//     // \theta = \arctan \left( \frac{v_0^2 \pm \sqrt{v_0^4 - g(gx_f^2+2y_fv_0^2)}}{gx_f} \right)
//     let base_angles = |range: f32, y_diff: f32| {
//         let v0_sq = speed * speed;
//         let par = gravity * range.powi(2) + 2.0 * y_diff * v0_sq;
//         let sub_radical = v0_sq * v0_sq - gravity * par;
//         if sub_radical < 0.0 {
//             (false, 0.0, 0.0)
//         } else {
//             let radical = (sub_radical).sqrt();
//             let _ang2 = ((v0_sq + radical) / (gravity * range)).atan();
//             let _ang1 = ((v0_sq - radical) / (gravity * range)).atan();
//             (true, _ang1, _ang2)
//         }
//     };
//     let (ok, mut _ang1, mut _ang2) = base_angles(range, y_diff);
//     if !ok {
//         return BulletSolutions {
//             low_sol: None,
//             high_sol: None,
//             err_sol: Some(make_solution(PI / 4.0, points)),
//         };
//     }

//     // https://www.lehman.edu/faculty/dgaranin/Mathematical_Physics/Mathematical_physics-10-Differential_equations.pdf
//     // δθi = αL / (v0 * Cos[2*ϑi] * Cos[ϑi])  * (gL / 3 v0^2)
//     let correct_angle = |ang: f32, range: f32| {
//         let t1 = (alpha * range) / (speed * ang.cos() * (2.0 * ang).cos());
//         let t2 = (gravity * range) / (3.0 * speed * speed);
//         t1 * t2
//     };
//     _ang1 += correct_angle(_ang1, range);
//     _ang2 += correct_angle(_ang2, range);
//     if _ang1 > _ang2 {
//         return BulletSolutions {
//             low_sol: None,
//             high_sol: None,
//             err_sol: Some(make_solution(PI / 4.0, points)),
//         };
//     }
//     BulletSolutions {
//         low_sol: Some(make_solution(_ang1, points)),
//         high_sol: Some(make_solution(_ang2, points)),
//         err_sol: None,
//     }
// }

// // TODO compute with lienar damping
// // https://www.lehman.edu/faculty/dgaranin/Mathematical_Physics/Mathematical_physics-10-Differential_equations.pdf
