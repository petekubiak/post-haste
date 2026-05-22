use post_haste::init_postmaster;

#[post_haste::payloads]
enum MyPayloads {
    Variant1,
    Variant2,
}

#[post_haste::payloads]
enum MyOtherPayloads {
    VariantA,
    VariantB,
}

#[derive(Copy, Clone)]
#[post_haste::addresses]
enum AgentAddresses {
    AgentA,
    AgentB,
}

init_postmaster!();

fn main() {}
