use std::thread;

fn bifurcate(r: f32, slice: &mut Vec<u32>) {
    let resy = slice.len();
    let mut xprev = 0.5;

    for step in 0..2_500_000 {
        let x = r * xprev * (1.0 - xprev);
        xprev = x;
        // Remove early steps as the equivalent of "burn in" in Monte Carlo
        if step > 100 {
            let idx = resy - (x * resy as f32) as usize;
            if idx < resy {
                slice[idx] = slice[idx] + 1;
            }
        }
    }

    // Normalize such that the strongest resulting pixel is pure white (255)
    let max: u32 = *slice.iter().max().unwrap();
    if max > 0 {
        slice.iter_mut().for_each(|x| *x = (255 * *x) / max);
    }
}

fn main() {
    const RESX: usize = 4 * 1080;
    const RESY: usize = 4 * 720;
    const STEP: usize = 8;

    const RANGEA: f32 = 2.4;
    const RANGEB: f32 = 4.0;

    eprintln!("Bifurcation Graph Plotter");
    eprintln!("=-=-=-=-=-=-=-=-=-=-=-=-=");

    let mut img: Vec<Vec<u32>> = Vec::new();

    // Scoped threads (i.e. http://aturon.github.io/crossbeam-doc/crossbeam/struct.Scope.html) would be helpful but are a dependency
    // History at https://users.rust-lang.org/t/why-does-thread-spawn-need-static-lifetime-for-generic-bounds/4541/2

    // Each step of the loop will farm work out to upwards of STEP threads
    for chunk in (0..RESX).collect::<Vec<usize>>().chunks(STEP) {
        // A status bar where \r resets output to the start of the line
        let percentage_complete = 100 * chunk.first().unwrap() / RESX;
        eprint!("\r[{}>{}]", "=".repeat(percentage_complete / 2), " ".repeat(50 - percentage_complete / 2));
        eprint!(" - ({}/{})", chunk.first().unwrap(), RESX);

        let mut threads = vec!();

        // Each thread creates a slice of the image, fills it using the `bifurcate` function, then returns the slice
        for x in chunk {
            // Ranging from RANGEA (usually 2.4) to RANGEB (usually 4.0)
            let r = RANGEA + (RANGEB - RANGEA) * (*x as f32 / RESX as f32);

            let handle = thread::spawn(move || {
                let mut slice: Vec<u32> = vec!(0; RESY);
                bifurcate(r, &mut slice);
                slice
            });
            threads.push(handle);
        }

        // Once all the threads are spawned we then retrieve the `slice` result from them in order
        // `into_iter` is necessary as it may yield (T, &T, &mut T) vs iter (&T) and JoinHandles are not Copy-able
        let slices = threads.into_iter().map(|x| x.join().unwrap());
        img.extend(slices);
    }
    // Print a new line (due to \r use above)
    eprintln!("");
    eprintln!("Completed calculations");

    eprint!("Writing Portable Graymap (PGM) to stdout...");
    // Print out a Portable Graymap (PGM) of the results
    // https://en.wikipedia.org/wiki/Netpbm#PGM_example
    // P2 notes grayscale, then XY resolution, with 255 being the max pixel value
    println!("P2");
    println!("{} {}", RESX, RESY);
    println!("255");
    for y in 0..RESY {
        for x in 0..RESX {
            // Bump up the bright for _any_ that have occurred
            let v = if img[x][y] > 0 { img[x][y] + 100 } else { 0 };
            // Limit the pixel to 255, the max range of our PGM
            let v = if v > 255 { 255 } else { v };
            print!("{} ", v)
        }
        println!("");
    }
    eprintln!(" Complete.\n");
}
