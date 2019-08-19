#include "wrapper.hpp"

#include <istream>
#include <sstream>
#include <iostream>

class membuf : public std::basic_streambuf<char>
{
public:
    membuf(const uint8_t *p, size_t l)
    {
        setg((char *)p, (char *)p, (char *)p + l);
    }
};

class memstream : public std::istream
{
public:
    memstream(const uint8_t *p, size_t l) : std::istream(&_buffer),
                                            _buffer(p, l)
    {
        rdbuf(&_buffer);
    }

private:
    membuf _buffer;
};

/*
cg3_init(stdin, stdout, stderr)
cg3_grammar_load_buffer(buff, size)
cg3_applicator_create(grammar)
cg3_run_grammar_on_text(applicator, istream, ostream)
cg3_grammar_free(grammar)
cg3_applicator_free(applicator)
cg3_cleanup()
*/

extern "C" std::stringstream *
cg3_run(const uint8_t *grammar_data, size_t grammar_size, const uint8_t *input_data, size_t input_size,
        size_t *output_size)
{
    if (!cg3_init(stdin, stdout, stderr))
        return nullptr;

    auto grammar = cg3_grammar_load_buffer((const char *)grammar_data, grammar_size);

    if (!grammar)
        return nullptr;

    auto applicator = cg3_applicator_create(grammar);

    if (!applicator)
        return nullptr;

    memstream input_stream(input_data, input_size);

    auto output = new std::stringstream(std::ios::in | std::ios::out | std::ios::binary);

    cg3_run_grammar_on_text(applicator, &input_stream, output);
    cg3_applicator_free(applicator);
    cg3_grammar_free(grammar);

    output->seekg(0, output->end);
    *output_size = output->tellg();

    return output;
}

extern "C" void cg3_free(std::stringstream *stream)
{
    delete stream;
}

extern "C" void cg3_copy_output(std::stringstream *stream, char *output, size_t size)
{
    stream->seekg(0, stream->beg);
    stream->read(output, size);
}
