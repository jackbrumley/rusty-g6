import { useEffect, useLayoutEffect, useRef, useState } from "preact/hooks";

interface DropdownProps<T extends string> {
  /// `null` shows `placeholder` until a value is known.
  value: T | null;
  placeholder: string;
  options: { id: T; name: string }[];
  onChange: (value: T) => void;
  disabled?: boolean;
  ariaLabel: string;
}

/// A ghost-style dropdown with its own list (a native `<select>` list is
/// drawn by the system and cannot be styled). Keyboard: arrows move, Enter
/// or Space chooses, Escape closes, a letter jumps to the next match.
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
  const [openUpwards, setOpenUpwards] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);
  const listRef = useRef<HTMLUListElement>(null);

  const selectedIndex = options.findIndex((option) => option.id === value);
  const selectedName = selectedIndex >= 0 ? options[selectedIndex].name : placeholder;

  const openList = () => {
    if (disabled) {
      return;
    }
    setHighlighted(Math.max(selectedIndex, 0));
    setOpen(true);
  };

  const choose = (index: number) => {
    setOpen(false);
    const option = options[index];
    if (option && option.id !== value) {
      onChange(option.id);
    }
  };

  // Close on a click anywhere else.
  useEffect(() => {
    if (!open) {
      return;
    }
    const closeOnOutsideClick = (event: MouseEvent) => {
      if (!rootRef.current?.contains(event.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", closeOnOutsideClick);
    return () => document.removeEventListener("mousedown", closeOnOutsideClick);
  }, [open]);

  // Open upwards when the list would not fit below.
  useLayoutEffect(() => {
    if (!open || !rootRef.current || !listRef.current) {
      return;
    }
    const button = rootRef.current.getBoundingClientRect();
    const spaceBelow = window.innerHeight - button.bottom;
    setOpenUpwards(spaceBelow < listRef.current.offsetHeight + 8 && button.top > spaceBelow);
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
        <span>{selectedName}</span>
        <span class={`dropdown-chevron ${open ? "open" : ""}`} aria-hidden="true" />
      </button>
      {open && (
        <ul class={`dropdown-list ${openUpwards ? "upwards" : ""}`} role="listbox" ref={listRef}>
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
              {option.name}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
