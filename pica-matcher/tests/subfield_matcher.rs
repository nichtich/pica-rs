use pica_matcher::subfield_matcher::{
    CardinalityMatcher, ExistsMatcher, InMatcher, Matcher,
    RegexMatcher, RelationMatcher, SubfieldMatcher,
};
use pica_matcher::MatcherOptions;
use pica_record::SubfieldRef;

#[test]
fn exists_matcher() -> anyhow::Result<()> {
    let matcher = ExistsMatcher::new("1?")?;
    let options = MatcherOptions::default();

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f1abc")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abc")?, &options));

    assert!(matcher.is_match(
        [
            &SubfieldRef::from_bytes(b"\x1f3def")?,
            &SubfieldRef::from_bytes(b"\x1f1hij")?,
        ],
        &options
    ));

    let matcher = ExistsMatcher::new("[a12]?")?;
    let options = MatcherOptions::default();

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f1abc")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f9abc")?, &options));
    assert!(matcher.is_match(
        [
            &SubfieldRef::from_bytes(b"\x1f3def")?,
            &SubfieldRef::from_bytes(b"\x1f9hij")?,
            &SubfieldRef::from_bytes(b"\x1f2bsg")?,
        ],
        &options
    ));

    Ok(())
}

#[test]
fn relational_matcher_eq() -> anyhow::Result<()> {
    // case sensitive
    let matcher = RelationMatcher::new("0 == 'abc'")?;
    let options = MatcherOptions::default();

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abc")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0ABC")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f1abc")?, &options));
    assert!(matcher.is_match(
        [
            &SubfieldRef::from_bytes(b"\x1f3def")?,
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &SubfieldRef::from_bytes(b"\x1f2bsg")?,
        ],
        &options
    ));

    // case insensitive
    let matcher = RelationMatcher::new("0 == 'abc'")?;
    let options = MatcherOptions::new().case_ignore(true);

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abc")?, &options));
    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0ABC")?, &options));

    // multiple subfields
    let matcher = RelationMatcher::new("0 == 'abc'")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        [
            &SubfieldRef::from_bytes(b"\x1f3def")?,
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &SubfieldRef::from_bytes(b"\x1f2bsg")?,
        ],
        &options
    ));

    assert!(!matcher.is_match(
        [
            &SubfieldRef::from_bytes(b"\x1f3def")?,
            &SubfieldRef::from_bytes(b"\x1f2bsg")?,
        ],
        &options
    ));

    Ok(())
}

#[test]
fn relational_matcher_ne() -> anyhow::Result<()> {
    // case sensitive
    let matcher = RelationMatcher::new("0 != 'abc'")?;
    let options = MatcherOptions::default();

    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abc")?, &options));
    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0ABC")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f1abc")?, &options));
    assert!(!matcher.is_match(
        [
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &SubfieldRef::from_bytes(b"\x1f2bsg")?,
        ],
        &options
    ));

    // case insensitive
    let matcher = RelationMatcher::new("0 != 'abc'")?;
    let options = MatcherOptions::new().case_ignore(true);

    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abc")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0ABC")?, &options));

    // multiple subfields
    let matcher = RelationMatcher::new("0 != 'abc'")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        [
            &SubfieldRef::from_bytes(b"\x1f3def")?,
            &SubfieldRef::from_bytes(b"\x1f0bsg")?,
        ],
        &options
    ));

    assert!(!matcher.is_match(
        [
            &SubfieldRef::from_bytes(b"\x1f3def")?,
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &SubfieldRef::from_bytes(b"\x1f2bsg")?,
        ],
        &options
    ));

    Ok(())
}

#[test]
fn relational_matcher_starts_with() -> anyhow::Result<()> {
    // case sensitive
    let matcher = RelationMatcher::new("0 =^ 'ab'")?;
    let options = MatcherOptions::default();

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abc")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0def")?, &options));

    // case insensitive
    let matcher = RelationMatcher::new("0 =^ 'ab'")?;
    let options = MatcherOptions::new().case_ignore(true);

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abc")?, &options));
    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0aBc")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0def")?, &options));

    // multiple subfields
    let matcher = RelationMatcher::new("0 =^ 'ab'")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        [
            &SubfieldRef::from_bytes(b"\x1f0baab")?,
            &SubfieldRef::from_bytes(b"\x1f0abba")?,
        ],
        &options
    ));

    assert!(!matcher.is_match(
        [
            &SubfieldRef::from_bytes(b"\x1f0def")?,
            &SubfieldRef::from_bytes(b"\x1f1abc")?,
        ],
        &options
    ));
    Ok(())
}

#[test]
fn relational_matcher_ends_with() -> anyhow::Result<()> {
    // case sensitive
    let matcher = RelationMatcher::new("0 =$ 'ab'")?;
    let options = MatcherOptions::default();

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abab")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abba")?, &options));

    // case insensitive
    let matcher = RelationMatcher::new("0 =$ 'ab'")?;
    let options = MatcherOptions::new().case_ignore(true);

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abab")?, &options));
    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abAB")?, &options));

    // multiple subfields
    let matcher = RelationMatcher::new("0 =$ 'ab'")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        [
            &SubfieldRef::from_bytes(b"\x1f0baba")?,
            &SubfieldRef::from_bytes(b"\x1f0abab")?,
        ],
        &options
    ));

    assert!(!matcher.is_match(
        [
            &SubfieldRef::from_bytes(b"\x1f0def")?,
            &SubfieldRef::from_bytes(b"\x1f1aab")?,
        ],
        &options
    ));
    Ok(())
}

#[test]
fn relational_matcher_similar() -> anyhow::Result<()> {
    // default threshold
    let matcher = RelationMatcher::new("a =* 'Heike'")?;
    let options = MatcherOptions::default();

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1faHeike")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1faHeiko")?, &options));

    // threshold set
    let matcher = RelationMatcher::new("a =* 'Heike'")?;
    let options = MatcherOptions::new().strsim_threshold(0.7);

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1faHeike")?, &options));
    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1faHeiko")?, &options));

    // default threshold
    let matcher = RelationMatcher::new("a =* 'Heike'")?;
    let options = MatcherOptions::new().case_ignore(true);

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1faheike")?, &options));

    // multiple subfields
    let matcher = RelationMatcher::new("a =* 'Heike'")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        vec![
            &SubfieldRef::from_bytes(b"\x1faHeiko")?,
            &SubfieldRef::from_bytes(b"\x1faHeike")?,
        ],
        &options
    ));

    Ok(())
}

#[test]
fn regex_matcher() -> anyhow::Result<()> {
    // case sensitive
    let matcher = RegexMatcher::new("0 =~ '^ab'")?;
    let options = MatcherOptions::default();

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abba")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0ABBA")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1faabba")?, &options));

    // case insensitive
    let matcher = RegexMatcher::new("0 =~ '^ab'")?;
    let options = MatcherOptions::new().case_ignore(true);

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abba")?, &options));
    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0ABBA")?, &options));

    // invert match
    let matcher = RegexMatcher::new("0 !~ '^ab'")?;
    let options = MatcherOptions::default();

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0baba")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abba")?, &options));

    // multiple subfields
    let matcher = RegexMatcher::new("0 =~ '^ab'")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        vec![
            &SubfieldRef::from_bytes(b"\x1f0foobar")?,
            &SubfieldRef::from_bytes(b"\x1f0abba")?
        ],
        &options
    ));

    assert!(!matcher.is_match(
        vec![
            &SubfieldRef::from_bytes(b"\x1f0foo")?,
            &SubfieldRef::from_bytes(b"\x1f0bar")?
        ],
        &options
    ));

    Ok(())
}

#[test]
fn in_matcher() -> anyhow::Result<()> {
    // case sensitive
    let matcher = InMatcher::new("0 in ['abc', 'def']")?;
    let options = MatcherOptions::default();

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abc")?, &options));
    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0def")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0hij")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0DEF")?, &options));

    // case insensitive
    let matcher = InMatcher::new("0 in ['abc', 'def']")?;
    let options = MatcherOptions::new().case_ignore(true);

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abc")?, &options));
    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0ABC")?, &options));

    // multiple subfields
    let matcher = InMatcher::new("0 in ['abc', 'def']")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        vec![
            &SubfieldRef::from_bytes(b"\x1f0hij")?,
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
        ],
        &options
    ));

    let matcher = InMatcher::new("a in ['000', '999']")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        vec![
            &SubfieldRef::from_bytes(b"\x1fa000")?,
            &SubfieldRef::from_bytes(b"\x1fzxyz")?,
        ],
        &options
    ));

    let matcher = InMatcher::new("a not in ['000', '999']")?;
    let options = MatcherOptions::default();

    assert!(!matcher.is_match(
        vec![
            &SubfieldRef::from_bytes(b"\x1fa000")?,
            &SubfieldRef::from_bytes(b"\x1fzxyz")?,
        ],
        &options
    ));

    Ok(())
}

#[test]
fn cardinality_matcher_eq() -> anyhow::Result<()> {
    let matcher = CardinalityMatcher::new("#0 == 2")?;
    let options = MatcherOptions::default();

    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fXabc"), &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abc"), &options));
    assert!(matcher.is_match(
        vec![
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &SubfieldRef::from_bytes(b"\x1f0def")?,
        ],
        &options
    ));
    assert!(!matcher.is_match(
        vec![
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &SubfieldRef::from_bytes(b"\x1f0def")?,
            &SubfieldRef::from_bytes(b"\x1f0hij")?,
        ],
        &options
    ));

    Ok(())
}

#[test]
fn cardinality_matcher_ne() -> anyhow::Result<()> {
    let matcher = CardinalityMatcher::new("#0 != 2")?;
    let options = MatcherOptions::default();

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fXabc"), &options));
    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abc"), &options));
    assert!(!matcher.is_match(
        vec![
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &SubfieldRef::from_bytes(b"\x1f0def")?,
        ],
        &options
    ));
    assert!(matcher.is_match(
        vec![
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &SubfieldRef::from_bytes(b"\x1f0def")?,
            &SubfieldRef::from_bytes(b"\x1f0hij")?,
        ],
        &options
    ));

    Ok(())
}

#[test]
fn cardinality_matcher_ge() -> anyhow::Result<()> {
    let matcher = CardinalityMatcher::new("#0 >= 2")?;
    let options = MatcherOptions::default();

    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fXabc"), &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abc"), &options));
    assert!(matcher.is_match(
        vec![
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &SubfieldRef::from_bytes(b"\x1f0def")?,
        ],
        &options
    ));
    assert!(matcher.is_match(
        vec![
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &SubfieldRef::from_bytes(b"\x1f0def")?,
            &SubfieldRef::from_bytes(b"\x1f0hij")?,
        ],
        &options
    ));

    Ok(())
}

#[test]
fn cardinality_matcher_gt() -> anyhow::Result<()> {
    let matcher = CardinalityMatcher::new("#0 > 2")?;
    let options = MatcherOptions::default();

    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fXabc"), &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abc"), &options));
    assert!(!matcher.is_match(
        vec![
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &SubfieldRef::from_bytes(b"\x1f0def")?,
        ],
        &options
    ));
    assert!(matcher.is_match(
        vec![
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &SubfieldRef::from_bytes(b"\x1f0def")?,
            &SubfieldRef::from_bytes(b"\x1f0hij")?,
        ],
        &options
    ));

    Ok(())
}

#[test]
fn cardinality_matcher_le() -> anyhow::Result<()> {
    let matcher = CardinalityMatcher::new("#0 <= 2")?;
    let options = MatcherOptions::default();

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fXabc"), &options));
    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abc"), &options));
    assert!(matcher.is_match(
        vec![
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &SubfieldRef::from_bytes(b"\x1f0def")?,
        ],
        &options
    ));
    assert!(!matcher.is_match(
        vec![
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &SubfieldRef::from_bytes(b"\x1f0def")?,
            &SubfieldRef::from_bytes(b"\x1f0hij")?,
        ],
        &options
    ));

    Ok(())
}

#[test]
fn cardinality_matcher_lt() -> anyhow::Result<()> {
    let matcher = CardinalityMatcher::new("#0 < 2")?;
    let options = MatcherOptions::default();

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fXabc"), &options));
    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1f0abc"), &options));
    assert!(!matcher.is_match(
        vec![
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &SubfieldRef::from_bytes(b"\x1f0def")?,
        ],
        &options
    ));
    assert!(!matcher.is_match(
        vec![
            &SubfieldRef::from_bytes(b"\x1f0abc")?,
            &SubfieldRef::from_bytes(b"\x1f0def")?,
            &SubfieldRef::from_bytes(b"\x1f0hij")?,
        ],
        &options
    ));

    Ok(())
}

#[test]
fn subfield_matcher_not() -> anyhow::Result<()> {
    // group
    let matcher = SubfieldMatcher::new("!(a == 'bcd')")?;
    let options = MatcherOptions::default();

    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fabcd")?, &options));
    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fbcde")?, &options));

    // exists
    let matcher = SubfieldMatcher::new("!a?")?;
    let options = MatcherOptions::default();

    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fabcd")?, &options));
    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fbcde")?, &options));

    // not
    let matcher = SubfieldMatcher::new("!!!(a == 'bcd')")?;
    let options = MatcherOptions::default();

    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fabcd")?, &options));
    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fbcde")?, &options));

    Ok(())
}

#[test]
fn subfield_matcher_group() -> anyhow::Result<()> {
    // and
    let matcher = SubfieldMatcher::new("(a =^ 'ab' && a =$ 'ba')")?;
    let options = MatcherOptions::default();

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1faabba")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fbcde")?, &options));

    // or
    let matcher = SubfieldMatcher::new("(a =^ 'ab' || a =^ 'ba')")?;
    let options = MatcherOptions::default();

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1faabba")?, &options));
    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fababa")?, &options));

    // singleton
    let matcher = SubfieldMatcher::new("(a == 'bcd')")?;
    let options = MatcherOptions::default();

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fabcd")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fbcde")?, &options));

    // nested group
    let matcher = SubfieldMatcher::new("(((a == 'bcd')))")?;
    let options = MatcherOptions::default();

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fabcd")?, &options));

    // not
    let matcher = SubfieldMatcher::new("(!(a == 'bcd'))")?;
    let options = MatcherOptions::default();

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fhijk")?, &options));

    Ok(())
}

#[test]
fn subfield_matcher_or() -> anyhow::Result<()> {
    // singleton
    let matcher =
        SubfieldMatcher::new("a =^ 'ab' || a =^ 'bc' || a =^ 'cd'")?;
    let options = MatcherOptions::default();

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1faabab")?, &options));
    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fabcbc")?, &options));
    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1facdcd")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fadede")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fbabab")?, &options));

    // group
    let matcher =
        SubfieldMatcher::new("a =^ 'ab' || (a =^ 'bc' && a =$ 'cd')")?;
    let options = MatcherOptions::default();

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1faabab")?, &options));
    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fabccd")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fabcbc")?, &options));

    // and
    let matcher =
        SubfieldMatcher::new("a =^ 'ab' || a =^ 'bc' && a =$ 'cd'")?;
    let options = MatcherOptions::default();

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1faabab")?, &options));
    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1faabcd")?, &options));
    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fabccd")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fabcbc")?, &options));

    // not
    let matcher = SubfieldMatcher::new("a == 'bcd' || !(a != 'def')")?;
    let options = MatcherOptions::default();

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fabcd")?, &options));
    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fadef")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fahij")?, &options));

    // boolean op precedence
    let matcher =
        SubfieldMatcher::new("(a =^ 'ab' || a =^ 'bc') && a =$ 'cd'")?;
    let options = MatcherOptions::default();

    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1faabab")?, &options));
    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1faabcd")?, &options));
    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fabccd")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fabcbc")?, &options));

    // multiple subfields
    let matcher = SubfieldMatcher::new("#a == 2 || a =^ 'ab'")?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        [
            &SubfieldRef::from_bytes(b"\x1fadef")?,
            &SubfieldRef::from_bytes(b"\x1fahij")?,
        ],
        &options
    ));

    assert!(matcher.is_match(
        [
            &SubfieldRef::from_bytes(b"\x1fadef")?,
            &SubfieldRef::from_bytes(b"\x1fahij")?,
            &SubfieldRef::from_bytes(b"\x1faabc")?,
        ],
        &options
    ));

    Ok(())
}

#[test]
fn subfield_matcher_and() -> anyhow::Result<()> {
    // singleton
    let matcher =
        SubfieldMatcher::new("#a == 1 && a =^ 'ab' && a =$ 'ba'")?;
    let options = MatcherOptions::default();

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1faabba")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fababa")?, &options));
    assert!(!matcher.is_match(
        [
            &SubfieldRef::from_bytes(b"\x1faabba")?,
            &SubfieldRef::from_bytes(b"\x1fababa")?,
        ],
        &options
    ));

    // group
    let matcher =
        SubfieldMatcher::new("#a == 1 && (a =^ 'ab' || a =^ 'ba')")?;
    let options = MatcherOptions::default();

    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1faabba")?, &options));
    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fababa")?, &options));
    assert!(!matcher.is_match(
        [
            &SubfieldRef::from_bytes(b"\x1faabba")?,
            &SubfieldRef::from_bytes(b"\x1fababa")?,
        ],
        &options
    ));

    // not
    let matcher =
        SubfieldMatcher::new("#a == 1 && !(a =^ 'ab' || a =^ 'ba')")?;
    let options = MatcherOptions::default();

    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1faabba")?, &options));
    assert!(!matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1fababa")?, &options));
    assert!(matcher
        .is_match(&SubfieldRef::from_bytes(b"\x1facbcb")?, &options));

    Ok(())
}
