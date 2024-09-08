use std::{
    collections::BTreeMap,
    env::args,
    fmt::{Display, Formatter, Result},
    fs::OpenOptions,
    io::{BufWriter, Write},
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Coord(i32, i32);

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Polyhex(Vec<Coord>);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PolyhexSymmetryGroup {
    None,
    Mirror0,
    Mirror30,
    Rotation2Fold,
    Rotation2FoldMirrorAll,
    Rotation3Fold,
    Rotation3FoldMirror0,
    Rotation3FoldMirror30,
    Rotation6Fold,
    All,
}

impl Coord {
    fn rotate(self) -> Coord {
        let Coord(x, y) = self;
        let previous_z_new_y = y;
        let previous_x_new_z = x - y;
        Coord(previous_x_new_z, previous_z_new_y + previous_x_new_z)
    }

    fn flip_x(self) -> Coord {
        let Coord(x, y) = self;
        let x0_mul_2 = y - 1;
        Coord(-x + x0_mul_2, y)
    }
}

impl Polyhex {
    fn rotate(&self) -> Polyhex {
        Polyhex(self.0.clone().into_iter().map(Coord::rotate).collect()).canonize_fixed()
    }

    fn flip_x(&self) -> Polyhex {
        Polyhex(self.0.clone().into_iter().map(Coord::flip_x).collect()).canonize_fixed()
    }

    fn canonize_fixed(&self) -> Polyhex {
        let min_x = self.0.iter().map(|coord| coord.0).min().unwrap_or(0);
        let min_y = self.0.iter().map(|coord| coord.1).min().unwrap_or(0);

        let mut normalized_coords: Vec<_> = self
            .0
            .iter()
            .map(|&Coord(x, y)| Coord(x - min_x, y - min_y))
            .collect();
        normalized_coords.sort();

        Polyhex(normalized_coords)
    }

    fn canonize_free(&self) -> (Polyhex, PolyhexSymmetryGroup) {
        let c0 = self.canonize_fixed();
        let c60 = c0.rotate();
        let c120 = c60.rotate();
        let c180 = c120.rotate();
        let c240 = c180.rotate();
        let c300 = c240.rotate();
        let f0 = c0.flip_x();
        let f60 = f0.rotate();
        let f120 = f60.rotate();
        let f180 = f120.rotate();
        let f240 = f180.rotate();
        let f300 = f240.rotate();

        let symmetry_group = if c0 == c60 {
            if c0 == f0 || c0 == f120 || c0 == f240 {
                PolyhexSymmetryGroup::All
            } else {
                PolyhexSymmetryGroup::Rotation6Fold
            }
        } else if c0 == c180 {
            if c0 == f0 || c0 == f120 || c0 == f240 {
                PolyhexSymmetryGroup::Rotation2FoldMirrorAll
            } else {
                PolyhexSymmetryGroup::Rotation2Fold
            }
        } else if c0 == f0 || c0 == f120 || c0 == f240 {
            if c0 == c120 {
                PolyhexSymmetryGroup::Rotation3FoldMirror30
            } else {
                PolyhexSymmetryGroup::Mirror30
            }
        } else if c0 == c120 {
            if c0 == c180 {
                PolyhexSymmetryGroup::Rotation3FoldMirror0
            } else {
                PolyhexSymmetryGroup::Rotation3Fold
            }
        } else if c0 == f180 || c0 == f300 || c0 == f60 {
            PolyhexSymmetryGroup::Mirror0
        } else {
            PolyhexSymmetryGroup::None
        };

        let mut all = vec![
            c0, c60, c120, c180, c240, c300, f0, f60, f120, f180, f240, f300,
        ];
        all.sort();

        (all.remove(0), symmetry_group)
    }
}

impl Display for Coord {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{},{}", self.0, self.1)
    }
}

impl Display for Polyhex {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        for (i, coord) in self.0.iter().enumerate() {
            if i != 0 {
                write!(f, ",")?;
            }
            coord.fmt(f)?;
        }
        Ok(())
    }
}

fn main() {
    let up_to = args().collect::<Vec<_>>()[1].parse::<usize>().unwrap();

    let mut previous = vec![Polyhex(vec![Coord(0, 0)])];

    for n in 2..(up_to + 1) {
        let current = previous
            .iter()
            .flat_map(|polyhex| {
                polyhex.0.iter().flat_map(|&Coord(x, y)| {
                    vec![
                        v(polyhex, Coord(x + 1, y)),
                        v(polyhex, Coord(x, y + 1)),
                        v(polyhex, Coord(x + 1, y + 1)),
                        v(polyhex, Coord(x - 1, y - 1)),
                        v(polyhex, Coord(x, y - 1)),
                        v(polyhex, Coord(x - 1, y)),
                    ]
                    .into_iter()
                    .flat_map(|x| x)
                })
            })
            .collect::<BTreeMap<Polyhex, PolyhexSymmetryGroup>>();
        save(n, &current);
        previous = current.into_iter().map(|(polyhex, _)| polyhex).collect();
    }

    fn v(polyhex: &Polyhex, new_coord: Coord) -> Vec<(Polyhex, PolyhexSymmetryGroup)> {
        if polyhex.0.iter().any(|&coord| coord == new_coord) {
            vec![]
        } else {
            let mut result = polyhex.0.clone();
            result.push(new_coord);
            vec![Polyhex(result).canonize_free()]
        }
    }

    fn save(n: usize, current: &BTreeMap<Polyhex, PolyhexSymmetryGroup>) {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true) // Truncate file if it already exists
            .open(format!("{}.json", n))
            .unwrap();
        let mut writer = BufWriter::new(file);

        write!(writer, "{{").unwrap();
        for (i, (polyhex, symmetry_group)) in current.iter().enumerate() {
            if i == 0 {
                write!(writer, "\n\t\"{}\": \"{:?}\"", polyhex, symmetry_group).unwrap();
            } else {
                write!(writer, ",\n\t\"{}\": \"{:?}\"", polyhex, symmetry_group).unwrap();
            }
        }
        write!(writer, "\n}}\n").unwrap();
        writer.flush().unwrap();
    }
}
