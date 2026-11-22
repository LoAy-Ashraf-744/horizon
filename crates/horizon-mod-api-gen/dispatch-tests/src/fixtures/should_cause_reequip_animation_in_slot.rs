/// A slot-aware reequip part: records the slot and the answer it arrived with, then answers its own.
struct SlotReequipProbe(bool, Arc<Mutex<Vec<(String, bool)>>>);

impl ItemPart for SlotReequipProbe {
    fn item_subscribed(&self) -> &'static [Option<HookId>] {
        &[Some(HookId::ShouldCauseReequipAnimationInSlot)]
    }
    fn should_cause_reequip_animation_in_slot(
        &self,
        _: &dyn ClientHost,
        event: &mut ShouldCauseReequipAnimationInSlotEvent<'_>,
    ) {
        self.1
            .lock()
            .unwrap()
            .push((event.slot.to_owned(), event.reequip));
        event.reequip = self.0;
    }
}

#[test]
fn slot_aware_reequip_parts_hear_the_slot_and_answer_in_turn() {
    install(Vec::<ModEntry>::new().into());
    let heard = Arc::new(Mutex::new(Vec::new()));
    install_parts(Parts {
        items: [true, false]
            .map(|answer| leak(SlotReequipProbe(answer, heard.clone())) as &dyn ItemPart)
            .into(),
        ..Default::default()
    });
    let stick = |count| ItemStack {
        item: "fixture:stick".into(),
        count,
    };
    let mut reequip = false;
    dispatch_should_cause_reequip_animation_in_slot(
        &TestHost,
        &stick(2),
        Some(&stick(1)),
        Hand::Off,
        "offhand",
        false,
        &mut reequip,
    );
    // The second part is given the first part's answer, and its own is the last word.
    assert!(!reequip);
    assert_eq!(
        *heard.lock().unwrap(),
        [("offhand".to_owned(), false), ("offhand".to_owned(), true)]
    );
    install_parts(Parts::default());
}
