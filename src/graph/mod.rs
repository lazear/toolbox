use crate::arena::typed::{Arena, Index};
use std::num::NonZeroU32;

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
    pub vertices: [VertexIndex; 2],
    pub next: [EdgeIndex; 2],
    data: E,
}

impl<E> Edge<E> {
    pub fn vertex(&self, dir: Direction) -> VertexIndex {
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
            edges: [EdgeIndex(None); 2],
            data,
        }
    }

    pub fn edge(&self, dir: Direction) -> EdgeIndex {
        self.edges[dir.index()]
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct EdgeIndex(Option<Index>);

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct VertexIndex(Index);

pub struct Graph<V, E> {
    arena: Arena<Vertex<V>>,
    edges: Arena<Edge<E>>,
}

impl<V, E> Graph<V, E> {
    pub fn with_capacity(cap: u32) -> Graph<V, E> {
        Graph {
            arena: Arena::with_capacity(cap as u32),
            edges: Arena::with_capacity(cap * 2),
        }
    }
    pub fn add_vertex(&mut self, data: V) -> VertexIndex {
        VertexIndex(self.arena.insert(Vertex::new(data)))
    }

    pub fn add_edge(&mut self, start: VertexIndex, end: VertexIndex, data: E) -> EdgeIndex {
        let edge = Edge {
            vertices: [start, end],
            next: [EdgeIndex(None); 2],
            data,
        };

        let idx = self.edges.insert(edge);
        let ret = EdgeIndex(Some(idx));

        let outgoing = std::mem::replace(&mut self.arena.get_mut(start.0).unwrap().edges[0], ret);
        let incoming = std::mem::replace(&mut self.arena.get_mut(end.0).unwrap().edges[1], ret);

        self.edges.get_mut(idx).unwrap().next = [outgoing, incoming];
        ret
    }

    pub fn edges(&self) -> impl Iterator<Item = &Edge<E>> {
        self.edges.iter()
    }

    pub fn vertices(&self) -> impl Iterator<Item = &Vertex<V>> {
        self.arena.iter()
    }

    pub fn get_vertex(&self, index: VertexIndex) -> Option<&Vertex<V>> {
        self.arena.get(index.0)
    }

    pub fn get_edge(&self, index: EdgeIndex) -> Option<&Edge<E>> {
        self.edges.get(index.0?)
    }
}
