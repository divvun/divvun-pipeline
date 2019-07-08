@0xed14912900a5589c;

struct ModuleMetadata {
    moduleName @0 :Text;
    moduleVersion @1 :Text;
    commands @2 :List(ModuleCommandMetadata);
}

struct ModuleCommandMetadata {
    name @0 :Text;
    inputs @1 :List(UInt64);
    output @2 :UInt64;
}
