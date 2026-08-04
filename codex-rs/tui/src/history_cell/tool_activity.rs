use super::*;
use crate::line_truncation::truncate_line_with_ellipsis_if_overflow;

#[derive(Debug)]
struct ToolActivityItem {
    id: Option<String>,
    cell: Box<dyn HistoryCell>,
}

#[derive(Debug)]
pub(crate) struct ToolActivityCell {
    items: Vec<ToolActivityItem>,
    complete: bool,
    start_time: Instant,
    animations_enabled: bool,
}

impl ToolActivityCell {
    pub(crate) fn new(
        id: Option<String>,
        cell: Box<dyn HistoryCell>,
        animations_enabled: bool,
    ) -> Self {
        Self {
            items: vec![ToolActivityItem { id, cell }],
            complete: false,
            start_time: Instant::now(),
            animations_enabled,
        }
    }

    pub(crate) fn push(&mut self, id: Option<String>, cell: Box<dyn HistoryCell>) {
        self.items.push(ToolActivityItem { id, cell });
    }

    pub(crate) fn upsert(&mut self, id: String, cell: Box<dyn HistoryCell>) {
        if let Some(item) = self
            .items
            .iter_mut()
            .rev()
            .find(|item| item.id.as_deref() == Some(id.as_str()))
        {
            item.cell = cell;
        } else {
            self.push(Some(id), cell);
        }
    }

    pub(crate) fn last_cell_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.items.last_mut()?.cell.as_any_mut().downcast_mut::<T>()
    }

    pub(crate) fn find_cell_mut<T: 'static>(
        &mut self,
        mut predicate: impl FnMut(&T) -> bool,
    ) -> Option<&mut T> {
        for item in self.items.iter_mut().rev() {
            if let Some(cell) = item.cell.as_any_mut().downcast_mut::<T>()
                && predicate(cell)
            {
                return Some(cell);
            }
        }
        None
    }

    pub(crate) fn complete(&mut self) {
        self.complete = true;
    }

    pub(crate) fn fail(&mut self) {
        self.items
            .iter_mut()
            .for_each(|item| item.cell.fail_activity());
        self.complete = true;
    }

    fn is_exploration(&self) -> bool {
        !self.items.is_empty()
            && self
                .items
                .iter()
                .all(|item| item.cell.is_exploration_activity())
    }

    fn is_active(&self) -> bool {
        if self.is_exploration() {
            self.items.iter().any(|item| item.cell.is_active_activity())
        } else {
            !self.complete
        }
    }

    fn compact_lines(&self, width: u16) -> Vec<Line<'static>> {
        let available = width.saturating_sub(4);
        self.items
            .iter()
            .flat_map(|item| item.cell.activity_lines(available))
            .map(strip_activity_marker)
            .collect()
    }
}

impl HistoryCell for ToolActivityCell {
    fn display_lines(&self, width: u16) -> Vec<Line<'static>> {
        let active = self.is_active();
        let title = match (self.is_exploration(), active) {
            (true, true) => "Exploring",
            (true, false) => "Explored",
            (false, true) => "Using tools",
            (false, false) => "Used tools",
        };
        let bullet = if active {
            activity_indicator(
                Some(self.start_time),
                MotionMode::from_animations_enabled(self.animations_enabled),
                ReducedMotionIndicator::StaticBullet,
            )
            .unwrap_or_else(|| "•".dim())
        } else {
            "•".dim()
        };
        let mut lines = vec![Line::from(vec![bullet, " ".into(), title.bold()])];
        for (index, line) in self.compact_lines(width).into_iter().enumerate() {
            let mut prefixed = Line::from(if index == 0 {
                "  └ ".dim()
            } else {
                "    ".into()
            });
            prefixed.extend(line);
            lines.push(truncate_line_with_ellipsis_if_overflow(
                prefixed,
                usize::from(width),
            ));
        }
        lines
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        for (index, item) in self.items.iter().enumerate() {
            if index > 0 {
                lines.push("".into());
            }
            lines.extend(item.cell.raw_lines());
        }
        lines
    }

    fn transcript_lines(&self, width: u16) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        for (index, item) in self.items.iter().enumerate() {
            if index > 0 {
                lines.push("".into());
            }
            lines.extend(item.cell.transcript_lines(width));
        }
        lines
    }

    fn transcript_animation_tick(&self) -> Option<u64> {
        if !self.animations_enabled || !self.is_active() {
            return None;
        }
        Some((self.start_time.elapsed().as_millis() / 50) as u64)
    }

    fn fail_activity(&mut self) {
        self.fail();
    }
}

fn strip_activity_marker(mut line: Line<'static>) -> Line<'static> {
    let first_is_marker = line.spans.first().is_some_and(|span| {
        let content = span.content.as_ref();
        content.trim().chars().count() == 1 && content.ends_with(' ')
    });
    if first_is_marker {
        line.spans.remove(0);
        return line;
    }
    let split_marker = line.spans.len() >= 2
        && line.spans[0].content.trim().chars().count() == 1
        && line.spans[1].content.as_ref() == " ";
    if split_marker {
        line.spans.drain(..2);
    }
    line
}
