@0x97ef828ddcb8f400;

struct PipelineError {
    enum ErrorKind {
        unknownCommand @0;
        parallelError @1;
        sequenceError @2;
        moduleError @3;
        invalidInput @4;
    }

    kind @0 :ErrorKind;
    message @1 :Text;
    source :union {
        noError @2 :Void;
        error @3 :PipelineError;
    }
}
