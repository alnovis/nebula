//! Per-article trend chart: one smoothed line per series over hourly or daily buckets.
//! The last bucket is still filling up: the line shows its count so far, and a dashed
//! segment shows where it ends if the rate of the last hour holds.

use std::fmt::Write;

use chrono::{DateTime, Duration, NaiveDateTime, Timelike, Utc};

// Same canvas as the daily views chart
const CL: f64 = 45.0;
const CR: f64 = 795.0;
const CT: f64 = 25.0;
const CB: f64 = 175.0;

pub struct Bucket {
    pub start: NaiveDateTime,
    pub readers: i64,
    pub halfway: i64,
}

pub struct Dot {
    pub x: String,
    pub y: String,
    pub title: String,
}

pub struct Segment {
    pub x1: String,
    pub y1: String,
    pub x2: String,
    pub y2: String,
}

pub struct GridLine {
    pub y: String,
    pub label: String,
}

pub struct Label {
    pub x: String,
    pub y: String,
    pub anchor: &'static str,
    pub text: String,
}

/// Text anchor that keeps a label at `x` inside the canvas
fn anchor(x: f64) -> &'static str {
    if x <= CL + 30.0 {
        "start"
    } else if x >= CR - 30.0 {
        "end"
    } else {
        "middle"
    }
}

#[derive(Default)]
pub struct TrendChart {
    pub readers_path: String,
    pub halfway_path: String,
    pub readers_projection: Vec<Segment>,
    pub halfway_projection: Vec<Segment>,
    pub dots: Vec<Dot>,
    pub grid_lines: Vec<GridLine>,
    pub x_labels: Vec<Label>,
    pub value_labels: Vec<Label>,
}

/// Bucket starts (UTC) covering `days`, oldest first; the last one holds `now`
pub fn bucket_starts(now: DateTime<Utc>, days: i32, hourly: bool) -> Vec<NaiveDateTime> {
    let now = now.naive_utc();
    let (current, step, n) = if hourly {
        (
            now.date().and_hms_opt(now.hour(), 0, 0).unwrap(),
            Duration::hours(1),
            days.max(1) * 24,
        )
    } else {
        (
            now.date().and_hms_opt(0, 0, 0).unwrap(),
            Duration::days(1),
            days.max(1),
        )
    };
    (0..n).rev().map(|i| current - step * i).collect()
}

/// Share of the bucket starting at `start` that has already passed
pub fn elapsed_share(now: DateTime<Utc>, start: NaiveDateTime, hourly: bool) -> f64 {
    let len = if hourly { 3600.0 } else { 86400.0 };
    ((now.naive_utc() - start).num_seconds() as f64 / len).clamp(0.0, 1.0)
}

/// Count at the end of the bucket if the last hour's rate holds for the rest of it
pub fn projection(so_far: i64, last_hour: i64, elapsed: f64, hourly: bool) -> f64 {
    let bucket_hours = if hourly { 1.0 } else { 24.0 };
    so_far as f64 + last_hour as f64 * (1.0 - elapsed) * bucket_hours
}

fn sign(v: f64) -> f64 {
    if v > 0.0 {
        1.0
    } else if v < 0.0 {
        -1.0
    } else {
        0.0
    }
}

/// Cubic Bezier segments through the points with Steffen's monotone tangents: the
/// curve never goes above or below its two neighbouring points
fn monotone_segments(pts: &[(f64, f64)]) -> Vec<[(f64, f64); 3]> {
    let n = pts.len();
    if n < 2 {
        return Vec::new();
    }
    let secant: Vec<f64> = pts
        .windows(2)
        .map(|w| (w[1].1 - w[0].1) / (w[1].0 - w[0].0))
        .collect();
    let mut m = vec![0.0; n];
    m[0] = secant[0];
    m[n - 1] = secant[n - 2];
    for i in 1..n - 1 {
        let (h0, h1) = (pts[i].0 - pts[i - 1].0, pts[i + 1].0 - pts[i].0);
        let (s0, s1) = (secant[i - 1], secant[i]);
        let p = (s0 * h1 + s1 * h0) / (h0 + h1);
        m[i] = (sign(s0) + sign(s1)) * s0.abs().min(s1.abs()).min(0.5 * p.abs());
    }
    (0..n - 1)
        .map(|i| {
            let h = (pts[i + 1].0 - pts[i].0) / 3.0;
            [
                (pts[i].0 + h, pts[i].1 + m[i] * h),
                (pts[i + 1].0 - h, pts[i + 1].1 - m[i + 1] * h),
                pts[i + 1],
            ]
        })
        .collect()
}

fn monotone_path(pts: &[(f64, f64)]) -> String {
    let Some(first) = pts.first() else {
        return String::new();
    };
    let mut d = format!("M{:.1},{:.1}", first.0, first.1);
    for [c1, c2, p] in monotone_segments(pts) {
        let _ = write!(
            d,
            " C{:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
            c1.0, c1.1, c2.0, c2.1, p.0, p.1
        );
    }
    d
}

/// Grid step of 1, 2 or 5 times a power of ten, about four lines up to `max_val`
fn grid_step(max_val: f64) -> f64 {
    let raw = max_val / 4.0;
    let magnitude = 10f64.powf(raw.log10().floor());
    let step = if raw / magnitude < 1.5 {
        magnitude
    } else if raw / magnitude < 3.5 {
        2.0 * magnitude
    } else {
        5.0 * magnitude
    };
    step.max(1.0)
}

/// Chart for zero-filled `buckets`, oldest first; the last bucket is the current one.
/// `last_hour` holds the readers and halfway readers of the last 60 minutes.
pub fn build(buckets: &[Bucket], last_hour: (i64, i64), elapsed: f64, hourly: bool) -> TrendChart {
    let n = buckets.len();
    if n < 2 {
        return TrendChart::default();
    }
    let (complete, last) = (&buckets[..n - 1], &buckets[n - 1]);
    let proj_readers = projection(last.readers, last_hour.0, elapsed, hourly);
    let proj_halfway = projection(last.halfway, last_hour.1, elapsed, hourly);

    let max_val = buckets
        .iter()
        .map(|b| b.readers as f64)
        .chain([proj_readers, 1.0])
        .fold(0.0, f64::max);
    let x = |i: usize| CL + i as f64 * (CR - CL) / (n - 1) as f64;
    let y = |v: f64| CB - v / max_val * (CB - CT);
    let pt = |v: f64| format!("{:.1}", v);

    let line = |f: fn(&Bucket) -> i64| {
        let pts: Vec<(f64, f64)> = buckets
            .iter()
            .enumerate()
            .map(|(i, b)| (x(i), y(f(b) as f64)))
            .collect();
        monotone_path(&pts)
    };
    let segment = |from: i64, to: f64| -> Vec<Segment> {
        vec![Segment {
            x1: pt(x(n - 2)),
            y1: pt(y(from as f64)),
            x2: pt(x(n - 1)),
            y2: pt(y(to)),
        }]
    };

    let label_fmt = if hourly {
        "%b %d %H:00"
    } else if n <= 14 {
        "%b %d"
    } else {
        "%m/%d"
    };
    let unit = if hourly { "hour" } else { "day" };
    let mut dots: Vec<Dot> = complete
        .iter()
        .enumerate()
        .map(|(i, b)| Dot {
            x: pt(x(i)),
            y: pt(y(b.readers as f64)),
            title: format!(
                "{}: {} readers, {} read halfway",
                b.start.format(label_fmt),
                b.readers,
                b.halfway
            ),
        })
        .collect();
    dots.push(Dot {
        x: pt(x(n - 1)),
        y: pt(y(last.readers as f64)),
        title: format!(
            "{} so far: {} readers, {} read halfway; at the last hour's rate ~{:.0} and ~{:.0} by the end of the {}",
            last.start.format(label_fmt),
            last.readers,
            last.halfway,
            proj_readers,
            proj_halfway,
            unit
        ),
    });

    let step = grid_step(max_val);
    let grid_lines = (1..)
        .map(|k| k as f64 * step)
        .take_while(|v| *v <= max_val)
        .map(|v| GridLine {
            y: pt(y(v)),
            label: format!("{:.0}", v),
        })
        .collect();

    let target = 6.min(n);
    let x_labels = (0..target)
        .map(|k| {
            let idx = k * (n - 1) / (target - 1);
            Label {
                x: pt(x(idx)),
                y: "193".into(),
                anchor: anchor(x(idx)),
                text: buckets[idx].start.format(label_fmt).to_string(),
            }
        })
        .collect();

    // Peak of the past buckets, then the current count and its projection; the two
    // share one label when they would overlap
    let mut value_labels = Vec::new();
    if let Some((i, b)) = complete
        .iter()
        .enumerate()
        .max_by_key(|(_, b)| b.readers)
        .filter(|(_, b)| b.readers > last.readers)
    {
        value_labels.push(Label {
            x: pt(x(i)),
            y: pt(y(b.readers as f64) - 6.0),
            anchor: anchor(x(i)),
            text: b.readers.to_string(),
        });
    }
    let (y_now, y_proj) = (y(last.readers as f64), y(proj_readers));
    let mut label_now = |at: f64, text: String| {
        value_labels.push(Label {
            x: pt(x(n - 1)),
            y: pt(at - 6.0),
            anchor: anchor(x(n - 1)),
            text,
        })
    };
    if (y_now - y_proj).abs() < 14.0 {
        label_now(
            y_now.min(y_proj),
            format!("{}, ~{:.0}", last.readers, proj_readers),
        );
    } else {
        label_now(y_now, last.readers.to_string());
        label_now(y_proj, format!("~{:.0}", proj_readers));
    }

    TrendChart {
        readers_path: line(|b| b.readers),
        halfway_path: line(|b| b.halfway),
        readers_projection: segment(complete[n - 2].readers, proj_readers),
        halfway_projection: segment(complete[n - 2].halfway, proj_halfway),
        dots,
        grid_lines,
        x_labels,
        value_labels,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_bucket_starts() {
        let now = Utc.with_ymd_and_hms(2026, 11, 10, 15, 20, 0).unwrap();

        let hours = bucket_starts(now, 2, true);
        assert_eq!(hours.len(), 48);
        assert_eq!(hours[47].to_string(), "2026-11-10 15:00:00");
        assert_eq!(hours[0].to_string(), "2026-11-08 16:00:00");

        let days = bucket_starts(now, 7, false);
        assert_eq!(days.len(), 7);
        assert_eq!(days[6].to_string(), "2026-11-10 00:00:00");
        assert_eq!(days[0].to_string(), "2026-11-04 00:00:00");
    }

    #[test]
    fn test_projection() {
        let now = Utc.with_ymd_and_hms(2026, 11, 10, 15, 20, 0).unwrap();
        let start = bucket_starts(now, 1, true)[23];
        let elapsed = elapsed_share(now, start, true);
        assert!((elapsed - 1.0 / 3.0).abs() < 1e-9);
        assert_eq!(projection(10, 6, elapsed, true).round(), 14.0);

        // A spike early in the day does not inflate the rest of it
        let start = bucket_starts(now, 1, false)[0];
        let elapsed = elapsed_share(now, start, false);
        assert_eq!(projection(520, 10, elapsed, false).round(), 607.0);
    }

    #[test]
    fn test_monotone_segments_stay_between_points() {
        let ys = [0.0, 0.0, 120.0, 30.0, 31.0, 5.0, 90.0, 90.0, 0.0];
        let pts: Vec<(f64, f64)> = ys
            .iter()
            .enumerate()
            .map(|(i, y)| (i as f64 * 10.0, *y))
            .collect();
        for (i, [c1, c2, _]) in monotone_segments(&pts).iter().enumerate() {
            let lo = pts[i].1.min(pts[i + 1].1);
            let hi = pts[i].1.max(pts[i + 1].1);
            for c in [c1, c2] {
                assert!(c.1 >= lo - 1e-9 && c.1 <= hi + 1e-9, "segment {i}: {c:?}");
            }
        }
    }

    #[test]
    fn test_build_draws_current_count_and_projection() {
        let now = Utc.with_ymd_and_hms(2026, 11, 10, 15, 30, 0).unwrap();
        let buckets: Vec<Bucket> = bucket_starts(now, 1, true)
            .into_iter()
            .map(|start| Bucket {
                start,
                readers: 4,
                halfway: 1,
            })
            .collect();

        let chart = build(&buckets, (40, 10), 0.5, true);
        assert!(chart.readers_path.contains(" 795.0,"));
        assert_eq!(chart.readers_projection.len(), 1);
        assert_eq!(chart.dots.len(), 24);
        let texts: Vec<&str> = chart.value_labels.iter().map(|l| l.text.as_str()).collect();
        assert_eq!(texts, ["4", "~24"]);

        // Projection close to the count: one shared label
        let chart = build(&buckets, (1, 0), 0.9, true);
        let texts: Vec<&str> = chart.value_labels.iter().map(|l| l.text.as_str()).collect();
        assert_eq!(texts, ["4, ~4"]);
    }
}
