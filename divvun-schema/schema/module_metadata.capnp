@0xed14912900a5589c;

struct ModuleMetadata {
    moduleName @0 :Text;
    commands @1 :List(ModuleCommandMetadata);
}

struct ModuleCommandMetadata {
    name @0 :Text;
    inputs @1 :List(UInt64);
    output @2 :UInt64;
}
