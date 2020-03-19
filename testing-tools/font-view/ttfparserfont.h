#pragma once

#include <QCoreApplication>
#include <QPainterPath>

#include <ttfparser.h>

#include "glyph.h"

class TtfParserFont
{
    Q_DECLARE_TR_FUNCTIONS(TtfParserFont)

public:
    TtfParserFont();
    ~TtfParserFont();

    void open(const QString &path, const quint32 index = 0);
    bool isOpen() const;

    FontInfo fontInfo() const;
    Glyph outline(const quint16 gid) const;

    QVector<VariationInfo> loadVariations();
    void setVariations(const QVector<Variation> &variations);

private:
    QByteArray m_fontData;
    ttfp_font *m_font = nullptr;
};
