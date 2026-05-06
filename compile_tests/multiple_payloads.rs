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

#[post_haste::addresses]
enum AgentAddresses {
    AgentA,
    AgentB,
}

post_haste::init_postmaster!();

fn main() {}
