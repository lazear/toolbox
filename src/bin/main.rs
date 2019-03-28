use toolbox;
use toolbox::arena::typed::{Arena, Index};
use toolbox::graph::{Direction::*, Edge, EdgeIndex, Graph, Vertex};

struct CharIter {
    start: char,
    end: char,
}

impl CharIter {
    fn new(start: char, end: char) -> CharIter {
        CharIter { start, end }
    }
}

impl Iterator for CharIter {
    type Item = char;
    fn next(&mut self) -> Option<Self::Item> {
        #[cfg(no_std)]
        use core::char;
        #[cfg(not(no_std))]
        use std::char;

        if self.start == self.end {
            None
        } else {
            let c = self.start;
            self.start = char::from_u32(self.start as u32 + 1)?;
            Some(c)
        }
    }
}

use std::collections::HashMap;

fn main() {
    let mut grph = Graph::with_capacity(32);
    let mut map = HashMap::new();
    let a = grph.add_vertex('a');
    map.insert('a', a);

    for v in CharIter::new('b', 'g') {
        let ix = grph.add_vertex(v);
        map.insert(v, ix);
        grph.add_edge(a, ix, ());
    }

    grph.add_edge(map[&'b'], map[&'f'], ());
    grph.add_edge(map[&'f'], map[&'c'], ());

    println!("digraph G {{");
    for v in grph.vertices() {
        let mut edge = grph.get_edge(v.edge(Outgoing));

        while let Some(e) = edge {
            let neighbor = grph.get_vertex(e.vertex(Incoming)).unwrap();
            println!("\t{} -> {}", v.data, neighbor.data);
            let out = e.next_edge(Outgoing);
            edge = grph.get_edge(out);
        }
    }
    println!("}}");

    let b = grph
        .vertices()
        .filter(|v| v.data == 'b')
        .collect::<Vec<&Vertex<char>>>()[0];
    println!("{:?}", b.edges);
}
