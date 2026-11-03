(function () {
  'use strict';

  var BASE = 'https://brumley.info/rusty-g6';

  var COMMANDS = {
    linux: {
      system: {
        label: 'System Install',
        command: 'curl -sf ' + BASE + '/install.sh | bash',
        hint: 'Open <strong>Terminal</strong> and run:',
        note: 'Installs the .deb (Debian, Ubuntu, Mint) or .rpm (Fedora) and asks for your password.',
      },
      appimage: {
        label: 'AppImage',
        command: 'curl -sf ' + BASE + '/install.sh | bash -s -- --appimage',
        hint: 'Open <strong>Terminal</strong> and run:',
        note: 'Installs to ~/.local/bin for any distro. Asks for your password once to let the app reach the G6 over USB.',
      },
      uninstall: {
        label: 'Uninstall',
        command: 'curl -sf ' + BASE + '/uninstall.sh | bash',
        hint: 'Open <strong>Terminal</strong> and run:',
        note: '',
      },
    },
    windows: {
      user: {
        label: 'User Install',
        command: 'irm ' + BASE + '/install.ps1 | iex',
        hint: 'Open <strong>PowerShell</strong> or <strong>Windows Terminal</strong> and run:',
        note: 'Installs just for you. No admin needed.',
      },
      system: {
        label: 'System Wide',
        command: 'irm ' + BASE + '/install-system.ps1 | iex',
        hint: 'Open <strong>PowerShell as Administrator</strong> and run:',
        note: 'Installs the MSI for every user on this PC.',
      },
      uninstall: {
        label: 'Uninstall',
        command: 'irm ' + BASE + '/uninstall.ps1 | iex',
        hint: 'Open <strong>PowerShell</strong> and run:',
        note: '',
      },
    },
  };

  var COPY_ICON =
    '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">' +
    '<rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>' +
    '<path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>';
  var CHECK_ICON =
    '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">' +
    '<polyline points="20 6 9 17 4 12"></polyline></svg>';

  var labels = document.querySelectorAll('[data-platform]');
  var labelsContainer = document.getElementById('platform-labels');
  var indicator = document.getElementById('platform-tab-indicator');
  var codeBlock = document.getElementById('install-code');
  var copyBtn = document.getElementById('install-copy-btn');
  var variants = document.getElementById('install-variants');
  var hint = document.getElementById('install-hint');
  var note = document.getElementById('install-note');

  function detectPlatform() {
    var ua = (navigator.userAgent || '').toLowerCase();
    var plat = ((navigator.userAgentData && navigator.userAgentData.platform) || navigator.platform || '').toLowerCase();
    if (plat.indexOf('win') !== -1 || ua.indexOf('windows') !== -1) return 'windows';
    return 'linux';
  }

  var platform = detectPlatform();
  var variant = Object.keys(COMMANDS[platform])[0];

  function updateIndicator() {
    var active = labelsContainer.querySelector('.platform-label.active');
    if (!active) { indicator.style.opacity = '0'; return; }
    var box = labelsContainer.getBoundingClientRect();
    var r = active.getBoundingClientRect();
    indicator.style.left = (r.left - box.left) + 'px';
    indicator.style.width = r.width + 'px';
    indicator.style.opacity = '1';
  }

  function render() {
    for (var i = 0; i < labels.length; i++) {
      var on = labels[i].getAttribute('data-platform') === platform;
      labels[i].classList.toggle('active', on);
      labels[i].setAttribute('aria-selected', on ? 'true' : 'false');
    }
    var options = COMMANDS[platform];
    if (!options[variant]) variant = Object.keys(options)[0];
    var v = options[variant];

    codeBlock.textContent = v.command;
    hint.innerHTML = v.hint;
    note.textContent = v.note;
    note.style.display = v.note ? '' : 'none';

    var html = '';
    var keys = Object.keys(options);
    for (var k = 0; k < keys.length; k++) {
      html += '<button type="button" class="install-variant-btn' + (keys[k] === variant ? ' active' : '') +
        '" data-variant="' + keys[k] + '">' + options[keys[k]].label + '</button>';
    }
    variants.innerHTML = html;
    var buttons = variants.querySelectorAll('[data-variant]');
    for (var b = 0; b < buttons.length; b++) {
      buttons[b].addEventListener('click', function (e) {
        variant = e.currentTarget.getAttribute('data-variant');
        render();
      });
    }
    updateIndicator();
  }

  for (var i = 0; i < labels.length; i++) {
    labels[i].addEventListener('click', function (e) {
      platform = e.currentTarget.getAttribute('data-platform');
      variant = Object.keys(COMMANDS[platform])[0];
      render();
    });
  }

  function copied() {
    copyBtn.innerHTML = CHECK_ICON;
    copyBtn.classList.add('copied');
    setTimeout(function () {
      copyBtn.innerHTML = COPY_ICON;
      copyBtn.classList.remove('copied');
    }, 1500);
  }

  function fallbackCopy(text) {
    var ta = document.createElement('textarea');
    ta.value = text;
    ta.style.position = 'fixed';
    ta.style.opacity = '0';
    document.body.appendChild(ta);
    ta.select();
    try { document.execCommand('copy'); copied(); } catch (e) { /* ignore */ }
    document.body.removeChild(ta);
  }

  copyBtn.innerHTML = COPY_ICON;
  copyBtn.addEventListener('click', function () {
    var text = codeBlock.textContent;
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(text).then(copied, function () { fallbackCopy(text); });
    } else {
      fallbackCopy(text);
    }
  });

  window.addEventListener('resize', updateIndicator);
  render();
})();
