#[derive(Debug)]
pub struct Node {
    id: [u8; 20],
    routing: RTable,
}

// Routing table
#[derive(Debug)]
pub struct RTable {}

#[cfg(test)]
mod tests {}
