use post_haste::init_postmaster;

#[post_haste::payloads]
enum MyPayloads {
    Variant1,
    Variant2,
}

// User hasn't used the addresses macro
#[derive(Copy, Clone)]
enum AgentAddresses {
    AgentA,
    AgentB,
}

init_postmaster!();

fn main() {}
