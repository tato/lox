#[derive(Clone)]
pub struct Sneakable<I: Iterator> {
    iter: I,
    previous: Option<I::Item>,
    peeked: Option<Option<I::Item>>,
    peeked_beyond: Option<Option<I::Item>>,
}

impl<I: Iterator> Sneakable<I> {
    pub fn new(iter: I) -> Sneakable<I> {
        Sneakable {
            iter,
            previous: None,
            peeked: None,
            peeked_beyond: None,
        }
    }
}

impl<I: Iterator> Iterator for Sneakable<I>
where
    I::Item: Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let next = match self.peeked.take() {
            Some(v) => v,
            None => self.iter.next(),
        };

        self.previous = next.clone();
        self.peeked = self.peeked_beyond.take();
        next
    }
}

impl<I: Iterator> Sneakable<I> {
    pub fn previous(&self) -> Option<&I::Item> {
        self.previous.as_ref()
    }

    pub fn peek(&mut self) -> Option<&I::Item> {
        let iter = &mut self.iter;
        self.peeked.get_or_insert_with(|| iter.next()).as_ref()
    }

    pub fn peek_next(&mut self) -> Option<&I::Item> {
        let iter = &mut self.iter;
        if self.peeked.is_none() {
            self.peeked = Some(iter.next());
        }
        self.peeked_beyond
            .get_or_insert_with(|| iter.next())
            .as_ref()
    }
}
