use crate::arena::typed::{Arena, Index};

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum Direction {
    Outgoing,
    Incoming,
}

impl Direction {
    pub fn index(self) -> usize {
        match self {
            Direction::Outgoing => 0,
            Direction::Incoming => 1,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Edge<E> {
    pub vertices: [Index; 2],
    pub next: [EdgeIndex; 2],
    data: E,
}

impl<E> Edge<E> {
    pub fn vertex(&self, dir: Direction) -> Index {
        self.vertices[dir.index()]
    }

    pub fn next_edge(&self, dir: Direction) -> EdgeIndex {
        self.next[dir.index()]
    }

    pub fn data(&self) -> &E {
        &self.data
    }
}

pub struct Vertex<V> {
    pub edges: [EdgeIndex; 2],
    pub data: V,
}

impl<V> Vertex<V> {
    fn new(data: V) -> Vertex<V> {
        Vertex {
            edges: [EdgeIndex::Empty; 2],
            data,
        }
    }

    pub fn edge(&self, dir: Direction) -> EdgeIndex {
        self.edges[dir.index()]
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum EdgeIndex {
    Empty,
    Edge(u32),
}

pub struct Graph<V, E> {
    arena: Arena<Vertex<V>>,
    edges: Vec<Edge<E>>,
}

impl<V, E> Graph<V, E> {
    pub fn with_capacity(cap: u32) -> Graph<V, E> {
        Graph {
            arena: Arena::with_capacity(cap as u32),
            edges: Vec::with_capacity(cap as usize * 2),
        }
    }
    pub fn add_vertex(&mut self, data: V) -> Index {
        self.arena.insert(Vertex::new(data))
    }

    pub fn add_edge(&mut self, start: Index, end: Index, data: E) -> EdgeIndex {
        let idx = EdgeIndex::Edge(self.edges.len() as u32);
        let outgoing = std::mem::replace(&mut self.arena.get_mut(start).unwrap().edges[0], idx);
        let incoming = std::mem::replace(&mut self.arena.get_mut(end).unwrap().edges[1], idx);
        let edge = Edge {
            vertices: [start, end],
            next: [outgoing, incoming],
            data,
        };

        self.edges.push(edge);
        idx
    }

    pub fn edges(&self) -> impl Iterator<Item = &Edge<E>> {
        self.edges.iter()
    }

    pub fn vertices(&self) -> impl Iterator<Item = &Vertex<V>> {
        self.arena.iter()
    }

    pub fn get_vertex(&self, index: Index) -> Option<&Vertex<V>> {
        self.arena.get(index)
    }

    pub fn get_edge(&self, index: EdgeIndex) -> Option<&Edge<E>> {
        match index {
            EdgeIndex::Edge(e) => self.edges.get(e as usize),
            EdgeIndex::Empty => None,
        }
    }
}
