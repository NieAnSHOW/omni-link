(function() {
  var EVENT_NAME = window.__EXTRACT_EVENT_NAME__ || 'extract-result';
  var hasInternals = !!(window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke);
  console.error('[extract.js] EVENT_NAME=' + EVENT_NAME + ' hasInternals=' + hasInternals);

  function emitResult(data) {
    if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke) {
      try {
        window.__TAURI_INTERNALS__.invoke('plugin:event|emit', {
          event: EVENT_NAME,
          payload: data
        }).then(function() {
          console.error('[extract.js] emit succeeded');
        }).catch(function(e) {
          console.error('[extract.js] emit failed: ' + e);
          fallbackNavigate(data);
        });
      } catch (e) {
        console.error('[extract.js] invoke error: ' + e.message);
        fallbackNavigate(data);
      }
    } else {
      console.error('[extract.js] no __TAURI_INTERNALS__, using fallback');
      fallbackNavigate(data);
    }
  }

  function fallbackNavigate(data) {
    try {
      var payload = Object.assign({}, data);
      delete payload.raw_html;
      var encoded = btoa(unescape(encodeURIComponent(JSON.stringify(payload))));
      window.location.href = 'omnilink-extract://done?data=' + encoded;
    } catch (e) {
      console.error('[extract.js] fallback encode error: ' + e.message);
      window.location.href = 'omnilink-extract://done?error=' + encodeURIComponent(e.message);
    }
  }

  try {
    var reader = new Readability(document.cloneNode(true));
    var article = reader.parse();

    if (!article || (!article.title && (!article.textContent || article.textContent.trim().length === 0))) {
      emitResult({
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

    emitResult({
      success: true,
      title: article.title || '',
      markdown: markdown,
      body_html: article.content || '',
      raw_html: rawHtml,
      metadata: metadata,
      images: Array.from(new Set(
        Array.from(document.querySelectorAll('img')).map(function(img) {
          return img.getAttribute('src') || '';
        }).filter(function(src) { return src.length > 0; })
      ))
    });
  } catch (e) {
    emitResult({
      success: false,
      error: e.message || 'Unknown extraction error'
    });
  }
})();
