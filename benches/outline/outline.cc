#include <fstream>
#include <streambuf>

#include <benchmark/benchmark.h>

#include <ft2build.h>
#include FT_FREETYPE_H
#include FT_OUTLINE_H

#include <ttfparser.h>

namespace FT {
struct Outliner
{
    static int moveToFn(const FT_Vector *, void *user)
    {
        auto self = static_cast<Outliner *>(user);
        self->counter += 1;
        return 0;
    }

    static int lineToFn(const FT_Vector *, void *user)
    {
        auto self = static_cast<Outliner *>(user);
        self->counter += 1;
        return 0;
    }

    static int quadToFn(const FT_Vector *, const FT_Vector *, void *user)
    {
        auto self = static_cast<Outliner *>(user);
        self->counter += 1;
        return 0;
    }

    static int cubicToFn(const FT_Vector *, const FT_Vector *, const FT_Vector *, void *user)
    {
        auto self = static_cast<Outliner *>(user);
        self->counter += 1;
        return 0;
    }

    uint32_t counter = 0;
};

class Font
{
public:
    Font(const std::string &path, const uint32_t index = 0)
    {
        if (FT_Init_FreeType(&m_library)) {
            throw "failed to init FreeType";
        }

        std::ifstream s(path);
        std::vector<char> data((std::istreambuf_iterator<char>(s)),
                                std::istreambuf_iterator<char>());
        m_fontData = std::move(data);

        if (FT_New_Memory_Face(m_library, (FT_Byte*)m_fontData.data(), m_fontData.size(), index, &m_face)) {
            throw "failed to open a font";
        }
    }

    ~Font()
    {
        if (m_face) {
            FT_Done_Face(m_face);
        }

        FT_Done_FreeType(m_library);
    }

    uint16_t numberOfGlyphs() const
    {
        return (uint16_t)m_face->num_glyphs;
    }

    uint32_t outline(const uint16_t gid) const
    {
        if (FT_Load_Glyph(m_face, gid, FT_LOAD_NO_SCALE | FT_LOAD_NO_BITMAP)) {
            throw "failed to load a glyph";
        }

        Outliner outliner;

        FT_Outline_Funcs funcs;
        funcs.move_to = outliner.moveToFn;
        funcs.line_to = outliner.lineToFn;
        funcs.conic_to = outliner.quadToFn;
        funcs.cubic_to = outliner.cubicToFn;
        funcs.shift = 0;
        funcs.delta = 0;

        if (FT_Outline_Decompose(&m_face->glyph->outline, &funcs, &outliner)) {
            throw "failed to outline a glyph";
        }

        return outliner.counter;
    }

private:
    std::vector<char> m_fontData;
    FT_Library m_library = nullptr;
    FT_Face m_face = nullptr;
};
}

namespace TTFP {
struct Outliner
{
    static void moveToFn(float x, float y, void *user)
    {
        auto self = static_cast<Outliner *>(user);
        self->counter += 1;
    }

    static void lineToFn(float x, float y, void *user)
    {
        auto self = static_cast<Outliner *>(user);
        self->counter += 1;
    }

    static void quadToFn(float x1, float y1, float x, float y, void *user)
    {
        auto self = static_cast<Outliner *>(user);
        self->counter += 1;
    }

    static void curveToFn(float x1, float y1, float x2, float y2, float x, float y, void *user)
    {
        auto self = static_cast<Outliner *>(user);
        self->counter += 1;
    }

    static void closePathFn(void *user)
    {
        auto self = static_cast<Outliner *>(user);
        self->counter += 1;
    }

    uint32_t counter = 0;
};

class Font
{
public:
    Font(const std::string &path, const uint32_t index = 0)
    {
        std::ifstream s(path);
        std::vector<char> data((std::istreambuf_iterator<char>(s)),
                                std::istreambuf_iterator<char>());
        m_fontData = std::move(data);

        m_font = ttfp_create_font((const uint8_t *)m_fontData.data(), m_fontData.size(), index);
        if (!m_font) {
            throw "failed to parse a font";
        }
    }

    ~Font()
    {
        if (m_font) {
            ttfp_destroy_font(m_font);
        }
    }

    uint16_t numberOfGlyphs() const
    {
        return ttfp_get_number_of_glyphs(m_font);
    }

    uint32_t outline(const uint16_t gid) const
    {
        Outliner outliner;

        ttfp_outline_builder builder;
        builder.move_to = outliner.moveToFn;
        builder.line_to = outliner.lineToFn;
        builder.quad_to = outliner.quadToFn;
        builder.curve_to = outliner.curveToFn;
        builder.close_path = outliner.closePathFn;

        ttfp_bbox bbox;
        ttfp_outline_glyph(m_font, builder, &outliner, gid, &bbox);

        return outliner.counter;
    }

private:
    std::vector<char> m_fontData;
    ttfp_font *m_font = nullptr;
};
}

static void freetype_outline_glyf(benchmark::State &state)
{
    FT::Font font("../fonts/SourceSansPro-Regular.ttf", 0);
    for (auto _ : state) {
        for (uint i = 0; i < font.numberOfGlyphs(); i++) {
            font.outline(i);
        }
    }
}
BENCHMARK(freetype_outline_glyf);

static void freetype_outline_cff(benchmark::State &state)
{
    FT::Font font("../fonts/SourceSansPro-Regular.otf", 0);
    for (auto _ : state) {
        for (uint i = 0; i < font.numberOfGlyphs(); i++) {
            font.outline(i);
        }
    }
}
BENCHMARK(freetype_outline_cff);

static void ttf_parser_outline_glyf(benchmark::State &state)
{
    TTFP::Font font("../fonts/SourceSansPro-Regular.ttf", 0);
    const auto numberOfGlyphs = font.numberOfGlyphs();
    for (auto _ : state) {
        for (uint i = 0; i < numberOfGlyphs; i++) {
            font.outline(i);
        }
    }
}
BENCHMARK(ttf_parser_outline_glyf);

static void ttf_parser_outline_cff(benchmark::State &state)
{
    TTFP::Font font("../fonts/SourceSansPro-Regular.otf", 0);
    const auto numberOfGlyphs = font.numberOfGlyphs();
    for (auto _ : state) {
        for (uint i = 0; i < numberOfGlyphs; i++) {
            font.outline(i);
        }
    }
}
BENCHMARK(ttf_parser_outline_cff);

BENCHMARK_MAIN();
