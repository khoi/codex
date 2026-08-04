use super::*;

impl ChatWidget {
    pub(super) fn push_tool_activity(
        &mut self,
        id: Option<String>,
        cell: impl HistoryCell + 'static,
    ) {
        self.push_boxed_tool_activity(id, Box::new(cell));
    }

    pub(super) fn push_boxed_tool_activity(
        &mut self,
        id: Option<String>,
        cell: Box<dyn HistoryCell>,
    ) {
        if let Some(group) = self
            .transcript
            .active_cell
            .as_mut()
            .and_then(|cell| cell.as_any_mut().downcast_mut::<ToolActivityCell>())
        {
            group.push(id, cell);
        } else {
            self.flush_active_cell();
            self.transcript.active_cell = Some(Box::new(ToolActivityCell::new(
                id,
                cell,
                self.config.animations,
            )));
        }
        self.bump_active_cell_revision();
        self.request_redraw();
    }

    pub(super) fn upsert_tool_activity(&mut self, id: String, cell: impl HistoryCell + 'static) {
        if let Some(group) = self
            .transcript
            .active_cell
            .as_mut()
            .and_then(|cell| cell.as_any_mut().downcast_mut::<ToolActivityCell>())
        {
            group.upsert(id, Box::new(cell));
        } else {
            self.flush_active_cell();
            self.transcript.active_cell = Some(Box::new(ToolActivityCell::new(
                Some(id),
                Box::new(cell),
                self.config.animations,
            )));
        }
        self.bump_active_cell_revision();
        self.request_redraw();
    }

    pub(super) fn active_tool_cell_mut<T: 'static>(
        &mut self,
        mut predicate: impl FnMut(&T) -> bool,
    ) -> Option<&mut T> {
        let active = self.transcript.active_cell.as_mut()?;
        if active.as_any().is::<ToolActivityCell>() {
            active
                .as_any_mut()
                .downcast_mut::<ToolActivityCell>()?
                .find_cell_mut(predicate)
        } else {
            let cell = active.as_any_mut().downcast_mut::<T>()?;
            predicate(cell).then_some(cell)
        }
    }

    pub(super) fn last_tool_cell_mut<T: 'static>(&mut self) -> Option<&mut T> {
        let active = self.transcript.active_cell.as_mut()?;
        if active.as_any().is::<ToolActivityCell>() {
            active
                .as_any_mut()
                .downcast_mut::<ToolActivityCell>()?
                .last_cell_mut::<T>()
        } else {
            active.as_any_mut().downcast_mut::<T>()
        }
    }
}
