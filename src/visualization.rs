/// This module visualizes the data using the plotters library.
// Required libraries
use plotters::prelude::*; // A plotting library for Rust
use std::collections::HashMap; // A collection type that stores key-value pairs
use std::error::Error; // A trait for error handling
use std::path::Path; // A type that represents a file path
use std::time::Duration; // A type that represents a span of time

pub fn plot_comparison(
    plaintext_results: &HashMap<String, f64>,
    encrypted_results: &HashMap<String, f64>,
    title: &str,
    output_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_value = plaintext_results
        .values()
        .chain(encrypted_results.values())
        .fold(0.0f64, |a, &b| a.max(b))
        * 1.2;

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 20).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..plaintext_results.len() as f64, 0.0..max_value)?;

    chart
        .configure_mesh()
        .x_labels(plaintext_results.len())
        .x_label_formatter(&|x| {
            plaintext_results
                .keys()
                .nth(*x as usize)
                .cloned()
                .unwrap_or_default()
        })
        .y_desc("Value")
        .draw()?;

    // Draw plaintext bars
    chart.draw_series(plaintext_results.values().enumerate().map(|(i, &value)| {
        let x0 = i as i32;
        let _x1 = x0 + 1;
        let bar_width = 0.3;

        Rectangle::new(
            [(x0 as f64 + 0.2, 0.0), (x0 as f64 + 0.2 + bar_width, value)],
            BLUE.filled(),
        )
    }))?;

    // Draw encrypted bars
    chart.draw_series(encrypted_results.values().enumerate().map(|(i, &value)| {
        let x0 = i as i32;
        let _x1 = x0 + 1;
        let bar_width = 0.3;

        Rectangle::new(
            [(x0 as f64 + 0.5, 0.0), (x0 as f64 + 0.5 + bar_width, value)],
            RED.filled(),
        )
    }))?;

    // Add legend
    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(0.0, 0.0), (0.3, 0.0)],
            BLUE,
        )))?
        .label("Plaintext")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(0.0, 0.0), (0.3, 0.0)],
            RED,
        )))?
        .label("Encrypted (FHE)")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));

    root.present()?;

    Ok(())
}

/// Creates a bar chart showing performance metrics
pub fn plot_performance_metrics(
    metrics: &HashMap<String, Duration>,
    title: &str,
    output_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_duration = metrics
        .values()
        .fold(Duration::from_secs(0), |a, &b| a.max(b));

    let max_secs = max_duration.as_secs_f64() * 1.2; // 20% margin

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 20).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..metrics.len() as f64, 0.0..max_secs)?;

    chart
        .configure_mesh()
        .x_labels(metrics.len())
        .x_label_formatter(&|x| metrics.keys().nth(*x as usize).cloned().unwrap_or_default())
        .y_desc("Time (seconds)")
        .draw()?;

    // Draw performance bars
    chart.draw_series(metrics.values().enumerate().map(|(i, &duration)| {
        let secs = duration.as_secs_f64();
        let x0 = i as i32;
        let _x1 = x0 + 1;
        let bar_width = 0.6;

        Rectangle::new(
            [(x0 as f64 + 0.2, 0.0), (x0 as f64 + 0.2 + bar_width, secs)],
            GREEN.filled(),
        )
    }))?;

    // Add data labels
    for (i, (_operation, &duration)) in metrics.iter().enumerate() {
        let secs = duration.as_secs_f64();
        let label = format!("{:.2}s", secs);

        let style = TextStyle::from(("sans-serif", 15).into_font()).color(&BLACK);

        root.draw_text(
            &label,
            &style,
            (
                ((i as f64 + 0.5) * 800.0 / metrics.len() as f64) as i32,
                (600.0 - (secs / max_secs * 500.0) - 20.0) as i32,
            ),
        )?;
    }

    root.present()?;

    Ok(())
}

/// Creates a visualization of the FHE workflow
pub fn visualize_fhe_workflow(output_path: &Path) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new(output_path, (1000, 700)).into_drawing_area();
    root.fill(&WHITE)?;

    // Define box positions
    let boxes = [
        // (label, x, y, width, height, color)
        (
            "Original\nBiosample Data",
            100,
            200,
            180,
            100,
            RGBColor(173, 216, 230),
        ), // Light blue
        (
            "Encrypted\nData",
            400,
            200,
            180,
            100,
            RGBColor(144, 238, 144),
        ), // Light green
        (
            "Homomorphic\nComputation",
            400,
            400,
            180,
            100,
            RGBColor(255, 255, 224),
        ), // Light yellow
        (
            "Encrypted\nResult",
            700,
            400,
            180,
            100,
            RGBColor(144, 238, 144),
        ), // Light green
        (
            "Decrypted\nResult",
            700,
            600,
            180,
            100,
            RGBColor(173, 216, 230),
        ), // Light blue
    ];

    // Draw boxes
    for &(label, x, y, width, height, color) in &boxes {
        // Draw box
        root.draw(&Rectangle::new(
            [(x, y), (x + width, y + height)],
            color.filled(),
        ))?;

        // Draw border
        root.draw(&Rectangle::new(
            [(x, y), (x + width, y + height)],
            BLACK.stroke_width(2),
        ))?;

        // Add label
        let style = TextStyle::from(("sans-serif", 15).into_font()).color(&BLACK);
        let lines: Vec<&str> = label.split('\n').collect();

        for (i, line) in lines.iter().enumerate() {
            let y_offset = if lines.len() > 1 {
                y + height / 2 - 10 + i as i32 * 20
            } else {
                y + height / 2
            };

            root.draw_text(line, &style.clone(), (x + width / 2, y_offset))?;
        }
    }

    // Define arrows
    let arrows = [
        // (start box index, end box index, label)
        (0, 1, "Encrypt"),
        (1, 2, "Process"),
        (2, 3, "Compute"),
        (3, 4, "Decrypt"),
    ];

    // Draw arrows
    for &(start_idx, end_idx, label) in &arrows {
        let (_start_label, start_x, start_y, start_w, start_h, _) = boxes[start_idx];
        let (_end_label, end_x, end_y, end_w, end_h, _) = boxes[end_idx];

        // Determine start and end points
        let (start_point_x, start_point_y, end_point_x, end_point_y) = if start_y == end_y {
            // Horizontal arrow
            (
                start_x + start_w,
                start_y + start_h / 2,
                end_x,
                end_y + end_h / 2,
            )
        } else if start_x == end_x {
            // Vertical arrow
            (
                start_x + start_w / 2,
                start_y + start_h,
                end_x + end_w / 2,
                end_y,
            )
        } else {
            // Diagonal arrow
            (
                start_x + start_w / 2,
                start_y + start_h,
                end_x + end_w / 2,
                end_y,
            )
        };

        // Draw arrow
        root.draw(&PathElement::new(
            vec![(start_point_x, start_point_y), (end_point_x, end_point_y)],
            BLACK.stroke_width(2),
        ))?;

        // Add arrowhead
        // (simplified - in practice you'd calculate this properly)
        let dx = end_point_x - start_point_x;
        let dy = end_point_y - start_point_y;
        let len = ((dx as f64) * (dx as f64) + (dy as f64) * (dy as f64)).sqrt();
        let nx = dx as f64 / len;
        let ny = dy as f64 / len;

        let arrow_size = 10.0;
        let arrow_x = end_point_x as f64;
        let arrow_y = end_point_y as f64;

        root.draw(&PathElement::new(
            vec![
                (arrow_x as i32, arrow_y as i32),
                (
                    (arrow_x - arrow_size * nx - arrow_size * ny * 0.5) as i32,
                    (arrow_y - arrow_size * ny + arrow_size * nx * 0.5) as i32,
                ),
                (
                    (arrow_x - arrow_size * nx + arrow_size * ny * 0.5) as i32,
                    (arrow_y - arrow_size * ny - arrow_size * nx * 0.5) as i32,
                ),
                (arrow_x as i32, arrow_y as i32),
            ],
            BLACK.filled(),
        ))?;

        // Add label
        let mid_x = (start_point_x + end_point_x) / 2;
        let mid_y = (start_point_y + end_point_y) / 2;

        let style = TextStyle::from(("sans-serif", 15).into_font()).color(&BLACK);

        // Draw label background
        root.draw(&Circle::new((mid_x, mid_y), 15, WHITE.filled()))?;

        root.draw_text(label, &style, (mid_x, mid_y))?;
    }

    // Add title
    let title_style = TextStyle::from(("sans-serif", 25, FontStyle::Bold)).color(&BLACK);

    root.draw_text(
        "Fully Homomorphic Encryption Workflow for Biosample Data",
        &title_style,
        (500, 50),
    )?;

    // Add notes
    let notes = [
        (150, 320, "Patient data\nremains private"),
        (850, 320, "Only computation results\nare revealed"),
        (500, 520, "All computations occur\non encrypted data"),
    ];

    for &(x, y, text) in &notes {
        // Draw note background
        root.draw(&Rectangle::new(
            [(x - 80, y - 25), (x + 80, y + 25)],
            RGBColor(255, 255, 224).filled(), // Light yellow
        ))?;

        root.draw(&Rectangle::new(
            [(x - 80, y - 25), (x + 80, y + 25)],
            BLACK.stroke_width(1),
        ))?;

        // Add note text
        let style = TextStyle::from(("sans-serif", 12).into_font()).color(&BLACK);
        let lines: Vec<&str> = text.split('\n').collect();

        for (i, line) in lines.iter().enumerate() {
            let y_offset = if lines.len() > 1 {
                y - 8 + i as i32 * 16
            } else {
                y
            };

            root.draw_text(line, &style.clone(), (x, y_offset))?;
        }
    }

    root.present()?;

    Ok(())
}
