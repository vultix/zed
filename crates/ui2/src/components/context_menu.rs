use crate::{
    h_stack, prelude::*, v_stack, KeyBinding, Label, List, ListItem, ListSeparator, ListSubHeader,
};
use gpui::{
    px, Action, AppContext, DismissEvent, Div, EventEmitter, FocusHandle, FocusableView,
    IntoElement, Render, View, VisualContext,
};
use menu::{SelectFirst, SelectLast, SelectNext, SelectPrev};
use std::rc::Rc;

pub enum ContextMenuItem {
    Separator,
    Header(SharedString),
    Entry {
        label: SharedString,
        handler: Rc<dyn Fn(&mut WindowContext)>,
        key_binding: Option<KeyBinding>,
    },
}

pub struct ContextMenu {
    items: Vec<ContextMenuItem>,
    focus_handle: FocusHandle,
    selected_index: Option<usize>,
}

impl FocusableView for ContextMenu {
    fn focus_handle(&self, _cx: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<DismissEvent> for ContextMenu {}

impl ContextMenu {
    pub fn build(
        cx: &mut WindowContext,
        f: impl FnOnce(Self, &mut WindowContext) -> Self,
    ) -> View<Self> {
        // let handle = cx.view().downgrade();
        cx.build_view(|cx| {
            f(
                Self {
                    items: Default::default(),
                    focus_handle: cx.focus_handle(),
                    selected_index: None,
                },
                cx,
            )
        })
    }

    pub fn header(mut self, title: impl Into<SharedString>) -> Self {
        self.items.push(ContextMenuItem::Header(title.into()));
        self
    }

    pub fn separator(mut self) -> Self {
        self.items.push(ContextMenuItem::Separator);
        self
    }

    pub fn entry(
        mut self,
        label: impl Into<SharedString>,
        on_click: impl Fn(&mut WindowContext) + 'static,
    ) -> Self {
        self.items.push(ContextMenuItem::Entry {
            label: label.into(),
            handler: Rc::new(on_click),
            key_binding: None,
        });
        self
    }

    pub fn action(
        mut self,
        label: impl Into<SharedString>,
        action: Box<dyn Action>,
        cx: &mut WindowContext,
    ) -> Self {
        self.items.push(ContextMenuItem::Entry {
            label: label.into(),
            key_binding: KeyBinding::for_action(&*action, cx),
            handler: Rc::new(move |cx| cx.dispatch_action(action.boxed_clone())),
        });
        self
    }

    pub fn confirm(&mut self, _: &menu::Confirm, cx: &mut ViewContext<Self>) {
        if let Some(ContextMenuItem::Entry { handler, .. }) =
            self.selected_index.and_then(|ix| self.items.get(ix))
        {
            (handler)(cx)
        }
        cx.emit(DismissEvent);
    }

    pub fn cancel(&mut self, _: &menu::Cancel, cx: &mut ViewContext<Self>) {
        cx.emit(DismissEvent);
    }

    fn select_first(&mut self, _: &SelectFirst, cx: &mut ViewContext<Self>) {
        self.selected_index = self.items.iter().position(|item| item.is_selectable());
        cx.notify();
    }

    fn select_last(&mut self, _: &SelectLast, cx: &mut ViewContext<Self>) {
        for (ix, item) in self.items.iter().enumerate().rev() {
            if item.is_selectable() {
                self.selected_index = Some(ix);
                cx.notify();
                break;
            }
        }
    }

    fn select_next(&mut self, _: &SelectNext, cx: &mut ViewContext<Self>) {
        if let Some(ix) = self.selected_index {
            for (ix, item) in self.items.iter().enumerate().skip(ix + 1) {
                if item.is_selectable() {
                    self.selected_index = Some(ix);
                    cx.notify();
                    break;
                }
            }
        } else {
            self.select_first(&Default::default(), cx);
        }
    }

    pub fn select_prev(&mut self, _: &SelectPrev, cx: &mut ViewContext<Self>) {
        if let Some(ix) = self.selected_index {
            for (ix, item) in self.items.iter().enumerate().take(ix).rev() {
                if item.is_selectable() {
                    self.selected_index = Some(ix);
                    cx.notify();
                    break;
                }
            }
        } else {
            self.select_last(&Default::default(), cx);
        }
    }
}

impl ContextMenuItem {
    fn is_selectable(&self) -> bool {
        matches!(self, Self::Entry { .. })
    }
}

impl Render for ContextMenu {
    type Element = Div;

    fn render(&mut self, cx: &mut ViewContext<Self>) -> Self::Element {
        div().elevation_2(cx).flex().flex_row().child(
            v_stack()
                .min_w(px(200.))
                .track_focus(&self.focus_handle)
                .on_mouse_down_out(cx.listener(|this, _, cx| this.cancel(&Default::default(), cx)))
                .key_context("menu")
                .on_action(cx.listener(ContextMenu::select_first))
                .on_action(cx.listener(ContextMenu::select_last))
                .on_action(cx.listener(ContextMenu::select_next))
                .on_action(cx.listener(ContextMenu::select_prev))
                .on_action(cx.listener(ContextMenu::confirm))
                .on_action(cx.listener(ContextMenu::cancel))
                .flex_none()
                .child(
                    List::new().children(self.items.iter().enumerate().map(
                        |(ix, item)| match item {
                            ContextMenuItem::Separator => ListSeparator.into_any_element(),
                            ContextMenuItem::Header(header) => {
                                ListSubHeader::new(header.clone()).into_any_element()
                            }
                            ContextMenuItem::Entry {
                                label: entry,
                                handler: callback,
                                key_binding,
                            } => {
                                let callback = callback.clone();
                                let dismiss = cx.listener(|_, _, cx| cx.emit(DismissEvent));

                                ListItem::new(entry.clone())
                                    .child(
                                        h_stack()
                                            .w_full()
                                            .justify_between()
                                            .child(Label::new(entry.clone()))
                                            .children(
                                                key_binding
                                                    .clone()
                                                    .map(|binding| div().ml_1().child(binding)),
                                            ),
                                    )
                                    .selected(Some(ix) == self.selected_index)
                                    .on_click(move |event, cx| {
                                        callback(cx);
                                        dismiss(event, cx)
                                    })
                                    .into_any_element()
                            }
                        },
                    )),
                ),
        )
    }
}
