/** Creates a `tag` element with `props` assigned to it and `children` appended. */
export function h<Tag extends keyof HTMLElementTagNameMap>(
  tag: Tag,
  props: Partial<HTMLElementTagNameMap[Tag]> = {},
  ...children: (Node | string)[]
): HTMLElementTagNameMap[Tag] {
  const element = Object.assign(document.createElement(tag), props);
  element.append(...children);
  return element;
}
