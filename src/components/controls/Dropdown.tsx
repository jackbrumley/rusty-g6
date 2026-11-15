import { createPortal } from "preact/compat";
import { useEffect, useLayoutEffect, useRef, useState } from "preact/hooks";

interface DropdownProps<T extends string> {
  /// `null` shows `placeholder` until a value is known.
  value: T | null;
  placeholder: string;
  /// An option's `description` shows under its name in the open list.
  options: { id: T; name: string; description?: string }[];
  onChange: (value: T) => void;
  disabled?: boolean;
  ariaLabel: string;
}

/// A dropdown with its own list (a native `<select>` list is drawn by the
/// system and cannot be styled). The open list renders in a portal at a fixed
/// position under its button, so a multi-column page cannot split it.
/// Keyboard: arrows move, Enter or Space chooses, Escape closes, a letter
/// jumps to the next match.
export function Dropdown<T extends string>({
  value,
  placeholder,
  options,
  onChange,
  disabled = false,
  ariaLabel,
}: DropdownProps<T>) {
  const [open, setOpen] = useState(false);
  const [highlighted, setHighlighted] = useState(0);
  const [listPosition, setListPosition] = useState<Record<string, string>>({});
  const rootRef = useRef<HTMLDivElement>(null);
  const listRef = useRef<HTMLUListElement>(null);

  const selectedIndex = options.findIndex((option) => option.id === value);
  const selectedName = selectedIndex >= 0 ? options[selectedIndex].name : placeholder;

  const openList = () => {
    if (disabled) {
      return;
    }
    setHighlighted(Math.max(selectedIndex, 0));
    setListPosition({});
    setOpen(true);
  };

  const choose = (index: number) => {
    setOpen(false);
    const option = options[index];
    if (option && option.id !== value) {
      onChange(option.id);
    }
  };

  // Close on a click anywhere else, and when the page scrolls or resizes
  // (the list is fixed to where its button was).
  useEffect(() => {
    if (!open) {
      return;
    }
    const closeOnOutsideClick = (event: MouseEvent) => {
      const target = event.target as Node;
      if (!rootRef.current?.contains(target) && !listRef.current?.contains(target)) {
        setOpen(false);
      }
    };
    const close = () => setOpen(false);
    // The list's own scrolling must not close it.
    const closeOnPageScroll = (event: Event) => {
      if (!listRef.current?.contains(event.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", closeOnOutsideClick);
    window.addEventListener("resize", close);
    document.addEventListener("scroll", closeOnPageScroll, true);
    return () => {
      document.removeEventListener("mousedown", closeOnOutsideClick);
      window.removeEventListener("resize", close);
      document.removeEventListener("scroll", closeOnPageScroll, true);
    };
  }, [open]);

  // Place the list under its button, or above it when it would not fit.
  useLayoutEffect(() => {
    if (!open || !rootRef.current || !listRef.current) {
      return;
    }
    const button = rootRef.current.getBoundingClientRect();
    const listHeight = listRef.current.offsetHeight;
    const spaceBelow = window.innerHeight - button.bottom;
    const upwards = spaceBelow < listHeight + 8 && button.top > spaceBelow;
    setListPosition({
      right: `${window.innerWidth - button.right}px`,
      minWidth: `${button.width}px`,
      ...(upwards
        ? { bottom: `${window.innerHeight - button.top + 6}px` }
        : { top: `${button.bottom + 6}px` }),
    });
  }, [open]);

  // Keep the highlighted option in view.
  useEffect(() => {
    if (open) {
      listRef.current?.children[highlighted]?.scrollIntoView({ block: "nearest" });
    }
  }, [open, highlighted]);

  const handleKeyDown = (event: KeyboardEvent) => {
    if (!open) {
      if (["ArrowDown", "ArrowUp", "Enter", " "].includes(event.key)) {
        event.preventDefault();
        openList();
      }
      return;
    }
    switch (event.key) {
      case "ArrowDown":
        event.preventDefault();
        setHighlighted((index) => Math.min(index + 1, options.length - 1));
        break;
      case "ArrowUp":
        event.preventDefault();
        setHighlighted((index) => Math.max(index - 1, 0));
        break;
      case "Home":
        event.preventDefault();
        setHighlighted(0);
        break;
      case "End":
        event.preventDefault();
        setHighlighted(options.length - 1);
        break;
      case "Enter":
      case " ":
        event.preventDefault();
        choose(highlighted);
        break;
      case "Escape":
      case "Tab":
        setOpen(false);
        break;
      default:
        if (event.key.length === 1) {
          const letter = event.key.toLowerCase();
          const next = options.findIndex(
            (option, index) => index > highlighted && option.name.toLowerCase().startsWith(letter)
          );
          const first = options.findIndex((option) => option.name.toLowerCase().startsWith(letter));
          const target = next >= 0 ? next : first;
          if (target >= 0) {
            setHighlighted(target);
          }
        }
    }
  };

  return (
    <div class="dropdown" ref={rootRef}>
      <button
        type="button"
        class="dropdown-button"
        aria-haspopup="listbox"
        aria-expanded={open}
        aria-label={ariaLabel}
        disabled={disabled}
        onClick={() => (open ? setOpen(false) : openList())}
        onKeyDown={handleKeyDown}
      >
        {/* Every name stacked in one cell, only the current one visible, so
            the button is as wide as its widest option and never resizes. */}
        <span class="dropdown-label">
          {[placeholder, ...options.map((option) => option.name)].map((name, index) => (
            <span key={index} class={name === selectedName ? "" : "dropdown-label-sizer"} aria-hidden={name !== selectedName}>
              {name}
            </span>
          ))}
        </span>
        <span class={`dropdown-chevron ${open ? "open" : ""}`} aria-hidden="true" />
      </button>
      {open &&
        createPortal(
          <ul
            class="dropdown-list"
            role="listbox"
            ref={listRef}
            // Hidden for the first layout pass, until it is placed.
            style={Object.keys(listPosition).length > 0 ? listPosition : { visibility: "hidden" }}
          >
            {options.map((option, index) => (
              <li
                key={option.id}
                role="option"
                aria-selected={index === selectedIndex}
                class={`dropdown-option ${index === highlighted ? "highlighted" : ""} ${
                  index === selectedIndex ? "selected" : ""
                }`}
                onMouseEnter={() => setHighlighted(index)}
                onMouseDown={(event) => event.preventDefault()}
                onClick={() => choose(index)}
              >
                <span class="dropdown-option-name">{option.name}</span>
                {option.description && <span class="dropdown-option-description">{option.description}</span>}
              </li>
            ))}
          </ul>,
          document.body
        )}
    </div>
  );
}
