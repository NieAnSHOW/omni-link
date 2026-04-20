(function() {
  try {
    var reader = new Readability(document.cloneNode(true));
    var article = reader.parse();

    if (!article || (!article.title && (!article.textContent || article.textContent.trim().length === 0))) {
      window.__TAURI__.event.emit('extract-result', {
        success: false,
        error: 'Readability failed to extract content'
      });
      return;
    }

    var turndownService = new TurndownService({
      headingStyle: 'atx',
      bulletListMarker: '-',
      codeBlockStyle: 'fenced'
    });
    var markdown = turndownService.turndown(article.content || '');

    var metadata = {
      description: article.excerpt || '',
      author: article.byline || '',
      site_name: article.siteName || '',
      image: '',
    };

    var ogImage = document.querySelector('meta[property="og:image"]');
    if (ogImage) metadata.image = ogImage.getAttribute('content') || '';

    var rawHtml = document.documentElement.outerHTML;

    window.__TAURI__.event.emit('extract-result', {
      success: true,
      title: article.title || '',
      markdown: markdown,
      body_html: article.content || '',
      raw_html: rawHtml,
      metadata: metadata,
      images: Array.from(document.querySelectorAll('img')).map(function(img) {
        return img.getAttribute('src') || '';
      }).filter(function(src) { return src.length > 0; })
    });
  } catch (e) {
    window.__TAURI__.event.emit('extract-result', {
      success: false,
      error: e.message || 'Unknown extraction error'
    });
  }
})();
