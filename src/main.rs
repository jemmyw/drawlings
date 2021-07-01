use array2d::Array2D;
use clap::{App, Arg};
use image::{DynamicImage, GenericImage, GenericImageView, ImageResult, Rgba};

//const OUT_FILE_NAME: &str = "plotters-doc-data/snowflake.png";
fn main() {
    let matches = App::new("drawlings")
        .version("1.0")
        .author("becker")
        .arg(
            Arg::new("INPUT")
                .about("input file")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("v")
                .short('v')
                .multiple_occurrences(true)
                .takes_value(true)
                .about("Sets the level of verbosity"),
        )
        .subcommand(
            App::new("vector_dump")
                .about("dump vectors from image")
                .version("0.0"),
        )
        .get_matches();

    // You can check the value provided by positional arguments, or option arguments
    let input = matches
        .value_of("INPUT")
        .expect("input file should always be present");

    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    let _verbose = matches.occurrences_of("v");

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level app
    if let Some(_matches) = matches.subcommand_matches("vector_dump") {
        let img = image::open(input).expect("could not open file");
        let grid = create_grid(&img);
        let path = create_path(&grid).unwrap();

        dbg!(&path.len());
        render_path(&grid, &path).unwrap();
        // render_grid(&grid).unwrap();
    }
}

type Pix = (u8, u8, u8, u8);
type PixGrid = Array2D<Pix>;
type Vertice = (usize, usize);
type Path = Vec<Vertice>;
type Dir = (i8, i8);

const DIR_NONE: Dir = (0, 0);
const DIR_UP: Dir = (0, -1);
const DIR_LEFT: Dir = (-1, 0);
const DIR_RIGHT: Dir = (1, 0);
const DIR_DOWN: Dir = (0, 1);

const EMPTY: Pix = (0, 0, 0, 0);
const FULL: Pix = (1, 1, 1, 1);
const L_T: Pix = (1, 1, 0, 0);

#[rustfmt::skip]
const L_B: Pix = (0, 0,
                  1, 1);
#[rustfmt::skip]
const L_R: Pix = (0, 1, 
                  0, 1);
#[rustfmt::skip]
const L_L: Pix = (1, 0,
                  1, 0);
#[rustfmt::skip]
const C_R_B: Pix = (1, 1, 
                    1, 0);

#[rustfmt::skip]
const C_L_B: Pix = (1, 1, 
                    0, 1);

#[rustfmt::skip]
const C_L_T: Pix = (0, 1, 
                    1, 1);
#[rustfmt::skip]
const C_R_T: Pix = (1, 0, 
                    1, 1);

#[rustfmt::skip]
const I_L_T: Pix = (1, 0, 
                    0, 0);
#[rustfmt::skip]
const I_R_T: Pix = (0, 1,
                    0, 0);
#[rustfmt::skip]
const I_R_B: Pix = (0, 0,
                    0, 1);
#[rustfmt::skip]
const I_L_B: Pix = (0, 0,
                    1, 0);
#[rustfmt::skip]
const D_L: Pix = (0, 1,
                  1, 0);
#[rustfmt::skip]
const D_R: Pix = (1, 0,
                  0, 1);

fn rot_left(pix: &Pix) -> Pix {
    (pix.1, pix.3, pix.0, pix.2)
}
fn rot_right(pix: &Pix) -> Pix {
    (pix.2, pix.0, pix.3, pix.1)
}
fn rot(pix: &Pix, dir: &Dir) -> Pix {
    match dir {
        &DIR_NONE => pix.clone(),
        &DIR_UP => pix.clone(),
        &DIR_LEFT => rot_right(pix),
        &DIR_RIGHT => rot_left(pix),
        &DIR_DOWN => rot_left(&rot_left(pix)),
        _ => {
            dbg!(dir);
            panic!("invalid direction")
        }
    }
}

const WHITE: Rgba<u8> = image::Rgba([255, 255, 255, 255]);

fn get_bw(img: &DynamicImage, x: u32, y: u32) -> u8 {
    if x >= img.width() || y >= img.height() {
        return 0;
    }

    if img.get_pixel(x, y) == WHITE {
        return 0;
    }

    return 1;
}

// Create the grid. The grid has 1 padding so that venturing into the edge returns FULL
fn create_grid(img: &DynamicImage) -> PixGrid {
    let width = (img.width()) as usize;
    let height = (img.height()) as usize;

    let mut grid = Array2D::filled_with(EMPTY, height + 2, width + 2);

    for (x, y, _rgba) in img.pixels() {
        let pix: Pix = (
            get_bw(img, x, y),
            get_bw(img, x + 1, y),
            get_bw(img, x, y + 1),
            get_bw(img, x + 1, y + 1),
        );

        grid.set((y + 1) as usize, (x + 1) as usize, pix).unwrap();
    }

    return grid;
}

fn get_start(grid: &PixGrid) -> Option<Vertice> {
    let mut x;
    let mut y = 0;

    for row in grid.rows_iter() {
        x = 0;
        for pix in row {
            if pix != &EMPTY && pix != &FULL {
                return Some((x, y));
            }
            x += 1;
        }
        y += 1;
    }

    return None;
}

fn turn_left(dir: &Dir) -> Dir {
    match dir {
        &DIR_NONE => DIR_LEFT,
        &DIR_DOWN => DIR_RIGHT,
        &DIR_LEFT => DIR_DOWN,
        &DIR_RIGHT => DIR_UP,
        &DIR_UP => DIR_LEFT,
        _ => {
            dbg!(dir);
            panic!("invalid direction")
        }
    }
}

fn turn_right(dir: &Dir) -> Dir {
    match dir {
        &DIR_NONE => DIR_RIGHT,
        &DIR_DOWN => DIR_LEFT,
        &DIR_LEFT => DIR_UP,
        &DIR_RIGHT => DIR_DOWN,
        &DIR_UP => DIR_RIGHT,
        _ => {
            dbg!(dir);
            panic!("invalid direction")
        }
    }
}

fn get_next_point(point: &Vertice, dir: &Dir) -> Option<Vertice> {
    let x: i64 = point.0 as i64 + dir.0 as i64;
    let y: i64 = point.1 as i64 + dir.1 as i64;
    if x < 0 || y < 0 {
        return None;
    }
    return Some((x as usize, y as usize));
}

fn get_next_direction(dir: &Dir, pix: &Pix) -> Dir {
    let rpix = rot(pix, dir);
    dbg!("rot", &rpix);

    match rpix {
        C_R_B => turn_right(dir),
        I_L_B => turn_left(dir),
        C_L_B => DIR_DOWN,
        C_L_T => DIR_LEFT,
        C_R_T => DIR_UP,
        I_L_T => DIR_UP,
        I_R_T => DIR_RIGHT,
        I_R_B => DIR_DOWN,
        D_L => turn_left(dir),
        D_R => turn_right(dir),
        _ => dir.clone(),
    }
}

fn get_pix<'a>(grid: &'a PixGrid, point: &Vertice) -> Option<&'a Pix> {
    grid.get(point.1, point.0)
}

fn create_path(grid: &PixGrid) -> Option<Path> {
    let first_point = get_start(grid)?;
    let mut point = first_point;
    let mut path: Path = vec![point];
    let pix = get_pix(&grid, &point)?;
    dbg!(&pix);

    let mut dir = match pix {
        &L_T => DIR_RIGHT,
        &L_B => DIR_LEFT,
        &L_R => DIR_DOWN,
        &L_L => DIR_UP,
        &C_R_B => DIR_RIGHT,
        &C_L_B => DIR_DOWN,
        &C_L_T => DIR_LEFT,
        &C_R_T => DIR_UP,
        &I_L_T => DIR_UP,
        &I_R_T => DIR_RIGHT,
        &I_R_B => DIR_DOWN,
        &I_L_B => DIR_LEFT,
        &D_L => DIR_LEFT,
        &D_R => DIR_UP,
        _ => {
            dbg!(pix);
            panic!("invalid pix")
        }
    };

    loop {
        point = get_next_point(&point, &dir)?;
        if point == first_point {
            dbg!("closed");
            break;
        }

        let pix = get_pix(&grid, &point)?;
        if pix == &FULL || pix == &EMPTY {
            dbg!("empty");
            break;
        }

        path.push(point);
        dbg!(&dir);
        dir = get_next_direction(&dir, &pix);
        dbg!(&pix);
        dbg!(&dir);
    }

    Some(path)
}

fn render_grid(grid: &PixGrid) -> ImageResult<()> {
    let mut img = DynamicImage::new_rgba8(grid.num_columns() as u32, grid.num_rows() as u32);
    let px_it = img.clone();

    for (x, y, _) in px_it.pixels() {
        let color = if grid.get(y as usize, x as usize).unwrap() == &EMPTY {
            Rgba([255, 255, 255, 255])
        } else {
            Rgba([0, 0, 0, 255])
        };

        img.put_pixel(x, y, color)
    }

    img.save_with_format("out.png", image::ImageFormat::Png)
}

fn render_path(grid: &PixGrid, path: &Path) -> ImageResult<()> {
    let mut img = DynamicImage::new_rgba8(grid.num_columns() as u32, grid.num_rows() as u32);
    let px_it = img.clone();

    for (x, y, _) in px_it.pixels() {
        img.put_pixel(x, y, Rgba([255, 255, 255, 255]))
    }

    for p in path {
        img.put_pixel(p.0 as u32, p.1 as u32, Rgba([0, 0, 0, 255]));
    }

    img.save_with_format("out.png", image::ImageFormat::Png)
}

// get the list of points along the line.
// fn generator_vec(img: &DynamicImage) -> Vec<Point> {
//     // i think i want to create a velocity for better tracking of which way to go..

//     // allow the points to be this many pixels away.
//     let pixel_movement = 1;

//     let mut return_points = vec![];

//     let direction_change = vec![
//         (pixel_movement, 0),
//         (pixel_movement, pixel_movement),
//         (pixel_movement.neg(), 0),
//         (pixel_movement.neg(), pixel_movement.neg()),
//     ];

//     let black_pixel = image::Rgba([0, 0, 0, 255]);

//     let (img_x, img_y) = img.dimensions();

//     // off by 1 for odd size images
//     let _middle = Point {
//         x: (img_x / 2) as i32,
//         y: (img_y / 2) as i32,
//     };

//     dbg!("here");
//     let first_spot = first_spot(img);

//     dbg!(&first_spot);
//     let mut current_spot = first_spot.expect("no black pixels located");
//     let first_spot = first_spot.unwrap();
//     return_points.push(current_spot);
//     'main_loop: loop {
//         let mut next = None;
//         'top_loop: for direct in &direction_change {
//             let move_x = direct.0 + current_spot.x;
//             let move_y = direct.1 + current_spot.y;
//             let point = Point {
//                 x: move_x,
//                 y: move_y,
//             };
//             if return_points.last().unwrap() == &point {
//                 continue;
//             }

//             let pixel = img.get_pixel(move_x as u32, move_y as u32);
//             if pixel == black_pixel {
//                 next = Some(point);
//                 break 'top_loop;
//             }
//         }
//         current_spot = next.expect("No next black pixel found");
//         dbg!(&return_points);
//         return_points.push(current_spot);
//         // dont check until we are far from the start
//         if return_points.len() > 10 && at_the_start(&current_spot, &first_spot)
//             || return_points.len() > 100_000
//         {
//             break 'main_loop;
//         }
//     }

//     return_points
// }
// Just get me any first spot. I thought it would work from the center
// but this was easier to reason about..
// fn first_spot(img: &DynamicImage) -> Option<Point> {
//     let (img_x, img_y) = img.dimensions();

//     let mut first_spot = None;
//     let black_pixel = image::Rgba([0, 0, 0, 255]);
//     'top_loop: while first_spot.is_none() {
//         for move_x in 0..img_x {
//             for move_y in 0..img_y {
//                 let pixel = img.get_pixel(move_x, move_y);
//                 if pixel == black_pixel {
//                     first_spot = Some(Point {
//                         x: move_x as i32,
//                         y: move_y as i32,
//                     });
//                     break 'top_loop;
//                 }
//             }
//         }
//     }
//     first_spot
// }
// let there be some buffer if we are back to the start.
// fn at_the_start(current: &Point, start: &Point) -> bool {
//     (current.x - start.x).abs() < 10 && (current.y - start.y).abs() < 5
// }

/*
fn display_image() -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(OUT_FILE_NAME, (1024, 768)).into_drawing_area();

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        //.caption("Koch's Snowflake", ("sans-serif", 50))
        .build_cartesian_2d(-200.0..200.0, -200.0..200.0)?;

    let mut snowflake_vertices = {
        let mut current: Vec<(f64, f64)> = vec![
            (100.0, 100.0),
            (-100.0, -100.0),
            (-100.0, 100.0),
            (199.0, -100.0),
        ];
        current
    };

    chart.draw_series(std::iter::once(Polygon::new(
        snowflake_vertices.clone(),
        &TRANSPARENT.mix(0.2),
    )))?;
    snowflake_vertices.push(snowflake_vertices[0]);
    chart.draw_series(std::iter::once(PathElement::new(
        snowflake_vertices,
        &BLACK,
    )))?;

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    println!("Result has been saved to {}", OUT_FILE_NAME);
    Ok(())
}

#[test]
fn entry_point() {
    main().unwrap()
}
*/
