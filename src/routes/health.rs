use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, Json};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
    database: &'static str,
    posts_count: usize,
    projects_count: usize,
}

pub async fn check(State(state): State<AppState>) -> Result<Json<HealthResponse>, StatusCode> {
    // Check database connection
    let db_status = sqlx::query("SELECT 1")
        .fetch_one(&state.pool)
        .await
        .map(|_| "ok")
        .unwrap_or("error");

    let content = state.content.read().await;

    Ok(Json(HealthResponse {
        status: "ok",
        database: db_status,
        posts_count: content.posts.len(),
        projects_count: content.projects.len(),
    }))
}

/// Diagnostic page for CDN availability from Russia
pub async fn russia_check() -> Html<&'static str> {
    Html(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>CDN Availability Check</title>
    <style>
        * { box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            background: #0f0f0f;
            color: #e0e0e0;
            padding: 2rem;
            max-width: 800px;
            margin: 0 auto;
        }
        h1 { color: #fff; }
        .cdn-list { list-style: none; padding: 0; }
        .cdn-item {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 1rem;
            margin: 0.5rem 0;
            background: #1a1a1a;
            border-radius: 8px;
            border-left: 4px solid #666;
        }
        .cdn-item.success { border-left-color: #22c55e; }
        .cdn-item.error { border-left-color: #ef4444; }
        .cdn-item.pending { border-left-color: #f59e0b; }
        .status {
            padding: 0.25rem 0.75rem;
            border-radius: 4px;
            font-size: 0.875rem;
            font-weight: 500;
        }
        .status.success { background: #22c55e20; color: #22c55e; }
        .status.error { background: #ef444420; color: #ef4444; }
        .status.pending { background: #f59e0b20; color: #f59e0b; }
        .timing { color: #888; font-size: 0.875rem; margin-left: 1rem; }
        .info { color: #888; margin-top: 2rem; font-size: 0.875rem; }
        .summary {
            margin-top: 2rem;
            padding: 1rem;
            background: #1a1a1a;
            border-radius: 8px;
        }
        button {
            background: #6366f1;
            color: white;
            border: none;
            padding: 0.75rem 1.5rem;
            border-radius: 6px;
            cursor: pointer;
            font-size: 1rem;
            margin-top: 1rem;
        }
        button:hover { background: #5558e3; }
        button:disabled { background: #444; cursor: not-allowed; }
    </style>
</head>
<body>
    <h1>CDN Availability Check</h1>
    <p>Testing CDN endpoints commonly blocked by DPI in Russia.</p>

    <ul class="cdn-list" id="results"></ul>

    <div class="summary" id="summary" style="display:none;">
        <strong>Summary:</strong> <span id="summary-text"></span>
    </div>

    <button id="run-btn" onclick="runTests()">Run Tests</button>
    <button id="report-btn" onclick="sendReport()" style="display:none;">Send Report to Server</button>

    <div class="info">
        <p>This page tests whether your browser can load resources from popular CDNs.</p>
        <p>If some CDNs fail, it may indicate DPI blocking by your ISP.</p>
    </div>

    <script>
    const CDN_TESTS = [
        { name: 'jsdelivr (HTMX)', url: 'https://cdn.jsdelivr.net/npm/htmx.org@1.9.10/dist/htmx.min.js' },
        { name: 'cdnjs (HTMX)', url: 'https://cdnjs.cloudflare.com/ajax/libs/htmx/1.9.10/htmx.min.js' },
        { name: 'unpkg (HTMX)', url: 'https://unpkg.com/htmx.org@1.9.10/dist/htmx.min.js' },
        { name: 'jsdelivr (Mermaid)', url: 'https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js' },
        { name: 'Cloudflare CDN', url: 'https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/highlight.min.js' },
        { name: 'Google Fonts', url: 'https://fonts.googleapis.com/css2?family=Inter:wght@400;500&display=swap' },
    ];

    let results = [];

    async function testCdn(cdn) {
        const start = performance.now();
        try {
            const controller = new AbortController();
            const timeout = setTimeout(() => controller.abort(), 10000);

            const response = await fetch(cdn.url, {
                method: 'HEAD',
                mode: 'no-cors',
                signal: controller.signal
            });

            clearTimeout(timeout);
            const timing = Math.round(performance.now() - start);

            // no-cors always returns opaque response, so we check via script loading
            return await testViaScript(cdn, start);
        } catch (e) {
            const timing = Math.round(performance.now() - start);
            return { ...cdn, success: false, timing, error: e.message };
        }
    }

    function testViaScript(cdn, start) {
        return new Promise((resolve) => {
            const script = document.createElement('script');
            const timeout = setTimeout(() => {
                script.remove();
                resolve({ ...cdn, success: false, timing: 10000, error: 'Timeout' });
            }, 10000);

            script.onload = () => {
                clearTimeout(timeout);
                script.remove();
                const timing = Math.round(performance.now() - start);
                resolve({ ...cdn, success: true, timing, error: null });
            };

            script.onerror = () => {
                clearTimeout(timeout);
                script.remove();
                const timing = Math.round(performance.now() - start);
                resolve({ ...cdn, success: false, timing, error: 'Load failed' });
            };

            script.src = cdn.url;
            document.head.appendChild(script);
        });
    }

    function renderResult(result, index) {
        const item = document.getElementById('cdn-' + index);
        item.className = 'cdn-item ' + (result.success ? 'success' : 'error');
        item.querySelector('.status').className = 'status ' + (result.success ? 'success' : 'error');
        item.querySelector('.status').textContent = result.success ? 'OK' : 'FAILED';
        item.querySelector('.timing').textContent = result.timing + 'ms';
    }

    async function runTests() {
        const btn = document.getElementById('run-btn');
        btn.disabled = true;
        btn.textContent = 'Testing...';

        const resultsEl = document.getElementById('results');
        resultsEl.innerHTML = CDN_TESTS.map((cdn, i) => `
            <li class="cdn-item pending" id="cdn-${i}">
                <div>
                    <strong>${cdn.name}</strong>
                    <div style="font-size:0.75rem;color:#666;margin-top:0.25rem;word-break:break-all;">${cdn.url}</div>
                </div>
                <div>
                    <span class="status pending">Testing...</span>
                    <span class="timing"></span>
                </div>
            </li>
        `).join('');

        results = [];
        for (let i = 0; i < CDN_TESTS.length; i++) {
            const result = await testViaScript(CDN_TESTS[i], performance.now());
            results.push(result);
            renderResult(result, i);
        }

        const success = results.filter(r => r.success).length;
        const total = results.length;

        const summaryEl = document.getElementById('summary');
        const summaryText = document.getElementById('summary-text');
        summaryEl.style.display = 'block';

        if (success === total) {
            summaryText.innerHTML = '<span style="color:#22c55e;">All CDNs accessible!</span> No DPI blocking detected.';
        } else if (success === 0) {
            summaryText.innerHTML = '<span style="color:#ef4444;">All CDNs blocked!</span> Severe connectivity issues.';
        } else {
            summaryText.innerHTML = `<span style="color:#f59e0b;">${success}/${total} CDNs accessible.</span> Partial blocking detected.`;
        }

        btn.textContent = 'Run Again';
        btn.disabled = false;
        document.getElementById('report-btn').style.display = 'inline-block';
    }

    async function sendReport() {
        const btn = document.getElementById('report-btn');
        btn.disabled = true;
        btn.textContent = 'Sending...';

        try {
            const response = await fetch('/health/russia/report', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    timestamp: new Date().toISOString(),
                    user_agent: navigator.userAgent,
                    results: results.map(r => ({
                        name: r.name,
                        url: r.url,
                        success: r.success,
                        timing_ms: r.timing,
                        error: r.error
                    }))
                })
            });

            if (response.ok) {
                btn.textContent = 'Report Sent!';
                btn.style.background = '#22c55e';
            } else {
                throw new Error('Server error');
            }
        } catch (e) {
            btn.textContent = 'Failed to send';
            btn.style.background = '#ef4444';
        }
    }
    </script>
</body>
</html>"#,
    )
}

#[derive(Debug, Deserialize)]
pub struct CdnReportRequest {
    timestamp: String,
    user_agent: String,
    results: Vec<CdnTestResult>,
}

#[derive(Debug, Deserialize)]
pub struct CdnTestResult {
    name: String,
    url: String,
    success: bool,
    timing_ms: u32,
    error: Option<String>,
}

#[derive(Serialize)]
pub struct ReportResponse {
    status: &'static str,
}

pub async fn russia_report(Json(report): Json<CdnReportRequest>) -> Json<ReportResponse> {
    let success_count = report.results.iter().filter(|r| r.success).count();
    let total = report.results.len();

    tracing::info!(
        "CDN Report: {}/{} accessible | UA: {} | Time: {}",
        success_count,
        total,
        report.user_agent,
        report.timestamp
    );

    for result in &report.results {
        if result.success {
            tracing::debug!("  [OK] {} - {}ms", result.name, result.timing_ms);
        } else {
            tracing::warn!(
                "  [FAIL] {} - {} ({}ms) - {}",
                result.name,
                result.error.as_deref().unwrap_or("unknown"),
                result.timing_ms,
                result.url
            );
        }
    }

    Json(ReportResponse { status: "ok" })
}
