extern crate image;
extern crate rand;

use rand::distributions::{IndependentSample, Range};
use std::fs::File;
use std::path::Path;

#[derive(Copy, Clone, Debug)]
struct Point2d{
    pub x: u32,
    pub y: u32
}
const IMGX: u32 = 2460;
const IMGY: u32 = 1300;

//layer 1 back (tall) .. layer 4 front (short)
const WHEEL_IN_THE_SKY: [u8; 4] = [255 as u8, 255 as u8, 255 as u8, 255 as u8];
const SKY_BG_COLOR: [u8; 4] = [240 as u8, 203 as u8, 163 as u8, 255 as u8];
const L1_COLOR: [u8; 4] = [157 as u8, 101 as u8, 202 as u8, 255 as u8];
const L2_COLOR: [u8; 4] = [129 as u8, 81 as u8, 137 as u8, 255 as u8];
const L3_COLOR: [u8; 4] = [68 as u8, 31 as u8, 98 as u8, 255 as u8];
const L4_COLOR: [u8; 4] = [49 as u8, 12 as u8, 81 as u8, 255 as u8];

const L1_DISP: f32 = IMGY as f32 * 0.58;
const L2_DISP: f32 = IMGY as f32 * 0.45;
const L3_DISP: f32 = IMGY as f32 * 0.35;
const L4_DISP: f32 = IMGY as f32 * 0.31;
//A: Starting points (left)  B: Ending points (right)
const L1_XA: u32 = 0;        const L1_YA: u32 = (IMGY as f32 * 0.25) as u32;
const L1_XB: u32 = IMGX-1;   const L1_YB: u32 = (IMGY as f32 * 0.27) as u32;
const L2_XA: u32 = 0;        const L2_YA: u32 = L1_YA + (IMGY as f32 * 0.15) as u32;
const L2_XB: u32 = IMGX-1;   const L2_YB: u32 = L1_YB + (IMGY as f32 * 0.15) as u32;
const L3_XA: u32 = 0;        const L3_YA: u32 = L2_YA + (IMGY as f32 * 0.13) as u32;
const L3_XB: u32 = 2*IMGX/3; const L3_YB: u32 = IMGY-1;
const L4_XA: u32 = IMGX/3;   const L4_YA: u32 = IMGY-1;
const L4_XB: u32 = IMGX-1;   const L4_YB: u32 = L2_YA + (IMGY as f32 * 0.13) as u32;
const W_X: u32 = IMGX/10;    const W_Y: u32 = IMGY/10;

const L1_ROUGHNESS: f32 = 0.89;
const L2_ROUGHNESS: f32 = 1.01;
const L3_ROUGHNESS: f32 = 1.45;
const L4_ROUGHNESS: f32 = 1.23;

const MAX_ITERATIONS: u32 = 10;
const WHEEL_RADIUS: i32 = IMGY as i32/17;
fn main() {
    // Create a new ImgBuf with width: imgx and height: imgy
    let mut imgbuf = image::ImageBuffer::new(IMGX, IMGY);

    //fill with background (sky) color
    for pixel in imgbuf.pixels_mut() {
        *pixel = image::Rgba(SKY_BG_COLOR);
    }
    //fill the WHEEL_IN_THE_SKY
    let wheel_center = Point2d{x: W_X, y: W_Y};
    //fill the center point
    if in_bounds(&wheel_center, IMGX, IMGY) {
        let mut pixel = imgbuf.get_pixel_mut(wheel_center.x, wheel_center.y);
        *pixel = image::Rgba(WHEEL_IN_THE_SKY);
    }
    for i in 1..WHEEL_RADIUS {
        let circle_points = get_circle_points(wheel_center, i);
        for point in circle_points.iter() {
            // println!("{:?}", point);
            if in_bounds(&point, IMGX, IMGY) {
                let mut pixel = imgbuf.get_pixel_mut(point.x, point.y);
                *pixel = image::Rgba(WHEEL_IN_THE_SKY);
            }
        }    
    }
    
    let start_end_points:[(Point2d, Point2d); 4] = [
                            (Point2d{x: L1_XA, y: L1_YA}, Point2d{x: L1_XB, y: L1_YB}),
                            (Point2d{x: L2_XA, y: L2_YA}, Point2d{x: L2_XB, y: L2_YB}),
                            (Point2d{x: L3_XA, y: L3_YA}, Point2d{x: L3_XB, y: L3_YB}),
                            (Point2d{x: L4_XA, y: L4_YA}, Point2d{x: L4_XB, y: L4_YB})];
    let layer_roughness = [L1_ROUGHNESS,
                           L2_ROUGHNESS,
                           L3_ROUGHNESS,
                           L4_ROUGHNESS];
    let layer_disp = [L1_DISP, L2_DISP, L3_DISP, L4_DISP];
    let layer_colors = [L1_COLOR, L2_COLOR, L3_COLOR, L4_COLOR];
    //modify the line using midpoint displacement
    for i in 0..start_end_points.len() {
        let layer_points = midpoint_displacement(start_end_points[i].0, start_end_points[i].1, 
                                                 layer_roughness[i], layer_disp[i], MAX_ITERATIONS);
        for j in 0..layer_points.len()-1{
            let point_a = *layer_points.get(j).expect("point oob");
            let point_b = *layer_points.get(j+1).expect("point oob");
            for point in &get_line_bres(point_a, point_b) {
                if in_bounds(&point, IMGX, IMGY) {
                    let mut pixel = imgbuf.get_pixel_mut(point.x, point.y);
                    *pixel = image::Rgba(layer_colors[i]);
                }
                for vert_point in &get_line_bres(Point2d{x: point.x, y: point.y+1}, Point2d{x: point.x, y: IMGY-1}) {
                    if in_bounds(&vert_point, IMGX, IMGY) {
                        let mut vert_pixel = imgbuf.get_pixel_mut(vert_point.x, vert_point.y);
                        *vert_pixel = image::Rgba(layer_colors[i]);
                    }
                }
            }
        }
    }
    let ref mut fout = File::create(&Path::new("terrain.png")).unwrap();
    //specify type and save image
    let _ = image::ImageRgba8(imgbuf).save(fout, image::PNG);
}
fn in_bounds(point: &Point2d, bound_x: u32, bound_y: u32) -> bool {
    point.x < bound_x && point.y < bound_y
}
//translated from roguebasin.com bresenham's line drawing algorithm
fn get_line_bres(a: Point2d, b: Point2d) -> Vec<Point2d> {
    let mut points = Vec::<Point2d>::new();
    let mut x1 = a.x as i32;
    let mut y1 = a.y as i32;
    let mut x2 = b.x as i32;
    let mut y2 = b.y as i32;
    let is_steep = (y2-y1).abs() > (x2-x1).abs();
    if is_steep {
        std::mem::swap(&mut x1, &mut y1);
        std::mem::swap(&mut x2, &mut y2);
    }
    let mut reversed = false;
    if x1 > x2 {
        std::mem::swap(&mut x1, &mut x2);
        std::mem::swap(&mut y1, &mut y2);   
        reversed = true;
    }
    let dx = x2 - x1;
    let dy = (y2 - y1).abs();
    let mut err = dx / 2;
    let mut y = y1;
    let ystep: i32;
    if y1 < y2 {
        ystep = 1;
    } else {
        ystep = -1;
    }
    for x in x1..(x2+1) {
        if is_steep {
            points.push(Point2d{x:y as u32, y:x as u32});
        } else {
            points.push(Point2d{x:x as u32, y:y as u32});
        }
        err -= dy;
        if err < 0 {
            y += ystep;
            err += dx;
        }
    }

    if reversed {
        for i in 0..(points.len()/2) {
            let end = points.len()-1;
            points.swap(i, end-i);
        }
    }
    points
}
// https://bitesofcode.wordpress.com/2016/12/23/landscape-generation-using-midpoint-displacement/
// Iterative midpoint vertical displacement
fn midpoint_displacement(a: Point2d, b: Point2d, roughness: f32, 
                         vertical_displacement: f32, iterations: u32) -> Vec<Point2d> {
    let mut vert_displ = vertical_displacement;
    let mut rng = rand::thread_rng();
    let mut points = vec!(a, b);
    for _ in 1..iterations {
        let mut temp_points = Vec::new();
        //go through the list of points
        for i in 0..points.len()-1{
            let point_a = *points.get(i).expect("point oob");
            let point_b = *points.get(i+1).expect("point oob");
            temp_points.push(point_a);
            //find midpoint of two points
            // println!("pt_a {:?} pt_b {:?}", point_a, point_b);
            let mut mid_point = Point2d{x:(point_a.x + point_b.x) / 2, y: (point_a.y + point_b.y) / 2};
            //displace the midpoint 
            let between = Range::new(-vert_displ, vert_displ);
            let sum = mid_point.y as f32 + between.ind_sample(&mut rng);
            if sum > 0.0 {
                mid_point.y = sum as u32;
            } else {
                mid_point.y = 0;
            }
            //insert into list
            temp_points.push(mid_point);
        }
        //always add the end back in
        temp_points.push(b);
        vert_displ *= (2.0 as f32).powf(-roughness);
        points = temp_points.to_vec();
    }
    points
}

fn get_circle_points(center: Point2d, radius: i32) -> Vec<Point2d> {
    let mut x: i32 = radius;
    let mut y: i32 = 0;
    let mut err: i32 = 0;
    let mut points = Vec::new();
    while x >=y {
        points.push(Point2d{x: center.x + x as u32, y: center.y + y as u32});
        points.push(Point2d{x: center.x + y as u32, y: center.y + x as u32});
        points.push(Point2d{x: center.x - y as u32, y: center.y + x as u32});
        points.push(Point2d{x: center.x - x as u32, y: center.y + y as u32});
        points.push(Point2d{x: center.x - x as u32, y: center.y - y as u32});
        points.push(Point2d{x: center.x - y as u32, y: center.y - x as u32});
        points.push(Point2d{x: center.x + y as u32, y: center.y - x as u32});
        points.push(Point2d{x: center.x + x as u32, y: center.y - y as u32});

        if err <= 0 {
            y +=1;
            err += 2 * y + 1;
        } else if err > 0 {
            x -= 1;
            err -= 2*x + 1;
        }
    }
//     int x = radius;
//     int y = 0;
//     int err = 0;

//     while (x >= y)
//     {
//         putpixel(x0 + x, y0 + y);
//         putpixel(x0 + y, y0 + x);
//         putpixel(x0 - y, y0 + x);
//         putpixel(x0 - x, y0 + y);
//         putpixel(x0 - x, y0 - y);
//         putpixel(x0 - y, y0 - x);
//         putpixel(x0 + y, y0 - x);
//         putpixel(x0 + x, y0 - y);

//         if (err <= 0)
//         {
//             y += 1;
//             err += 2*y + 1;
//         }
//         if (err > 0)
//         {
//             x -= 1;
//             err -= 2*x + 1;
//         }
//     }
    points
}
