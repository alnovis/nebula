// External fallback - only runs if inline scripts didn't execute

// HTMX with CDN fallback
if (typeof htmx === 'undefined' && !window._htmxLoading) {
    window._htmxLoading = true;
    (function() {
        var cdns = [
            'https://cdn.jsdelivr.net/npm/htmx.org@1.9.10/dist/htmx.min.js',
            'https://cdnjs.cloudflare.com/ajax/libs/htmx/1.9.10/htmx.min.js',
            'https://unpkg.com/htmx.org@1.9.10/dist/htmx.min.js'
        ];
        var loaded = false;

        function loadScript(url, callback) {
            var script = document.createElement('script');
            script.src = url;
            script.onload = function() { loaded = true; callback && callback(); };
            script.onerror = callback;
            document.head.appendChild(script);
        }

        function tryNext(index) {
            if (loaded || index >= cdns.length) return;
            loadScript(cdns[index], function() {
                if (!loaded) tryNext(index + 1);
            });
        }

        tryNext(0);
    })();
}

// Mermaid (lazy, only if needed)
if (!window._mermaidLoading && document.querySelector('.mermaid, pre.mermaid')) {
    window._mermaidLoading = true;
    import('https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.esm.min.mjs')
        .then(function(m) {
            m.default.initialize({
                startOnLoad: true,
                theme: 'dark',
                themeVariables: {
                    fontSize: '16px',
                    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
                    primaryColor: '#6366f1',
                    primaryTextColor: '#e0e0e0',
                    primaryBorderColor: '#6366f1',
                    lineColor: '#888',
                    secondaryColor: '#1a1a1a',
                    tertiaryColor: '#2a2a2a',
                    background: '#0f0f0f',
                    mainBkg: '#1a1a1a',
                    nodeBorder: '#6366f1',
                    clusterBkg: '#1a1a1a',
                    edgeLabelBackground: '#1a1a1a'
                },
                flowchart: {
                    nodeSpacing: 50,
                    rankSpacing: 50,
                    curve: 'basis',
                    padding: 15
                }
            });
            m.default.run();
        })
        .catch(function() { console.warn('Mermaid failed to load'); });
}

// Back to top button
if (!window._backToTopInit) {
    window._backToTopInit = true;
    var backToTop = document.querySelector('.back-to-top');
    if (backToTop) {
        window.addEventListener('scroll', function() {
            backToTop.classList.toggle('visible', window.scrollY > 300);
        });
    }
}

// Image error fallback
if (!window._imgFallbackInit) {
    window._imgFallbackInit = true;
    document.querySelectorAll('.post-cover img, .project-cover img, .post-hero img').forEach(function(img) {
        img.onerror = function() {
            this.style.display = 'none';
        };
    });
}
