Feature: Basic overview

Feature: Pipeline
  Scenario: Text goes in, text goes out
    Given an input string
    When a pipeline runs that reverses the text
    Then the text is output reversed

  Scenario: Image goes in, something comes out
    Given an image input
    When an OCR module is applied to the image
    Then the text is output

Feature: Cap'n Proto Definitions
  Scenario: We need to define these things

Feature: Modules
  Modules are loaded as dynamic libraries and expose a set of functions to be called
  by the pipeline runner.

  Scenario: A module is loaded
    Given a module
    When the module is loaded
    Then the initialiser function is called
    And it receives an allocator function

  Scenario: A module is unloaded
    Given a loaded module
    When the module is unloaded
    Then the deinitialise function is called
    And memory allocated by the module is freed

  Scenario: A module receives a list of sized inputs
    Given a loaded module
    When the module receives input
    Then it receives a list of input pointers
    And it receives a list of input sizes

  Scenario: A module returns output
    Given a loaded module
    When the module receives input
    Then it returns a pointer to output data
    And it returns the size of the output data

  Scenario: A module receives an invalid command
    Given a loaded module
    When the module receives a command it does not recognize
    Then an UnknownCommand error is returned

  Scenario: A module receives invalid input
    Given a loaded module
    When the module receives input it does not know how to process
    Then an InvalidInput error is returned


Feature: Binding to C/C++ codebases for modules

Feature: iOS static builds

Feature: Module: hfst3
  hfst3 stands for Helsinki Finite State Technology, and is a suite of tools for
  parsing and handling human text, such as tokenization.
  URL: https://github.com/hfst/hfst

  Scenario: Tokenise text input
    Given text input
    When the hfst module tokenises the text
    Then tokenised text is output

  Scenario: Blank tagging input
    Given tokenized text input
    When the hfst module tags the blanks in the given input
    # TODO: it is unclear whether this output will be the same type as the previous tokenised output
    Then tokenised text is output



Feature: Module: cg3
  cg3 stands for VISL CG3, a constraint grammar tool written in C++.
  URL: http://visl.sdu.dk/cg3.html

  Scenario: