# Tasktree

This tool shows your [taskwarrior](https://taskwarrior.org) tasks in a tree by introducing a new `partof` uda.

## Features

* Showing arbitrary queries to taskwarrior (refresh by hitting enter)
* Showing task properties: `description, status, tags, project, due, wait, entry, modified`
* Sort your tasks in a tree structure with drag and drop
* Delete/Undelete by setting checkbox
* Complete/Uncomplete by setting checkbox
* Create new task by filling new child field
* Edit task descriptions

I do not promise that there will come any more features. But a lot comes to mind. Pull requests welcome.

## Install
### Build Requirements

* rust
* cargo

### Run Requirements

* taskwarrior >= 2.5
* gtk >=3.22

### Install

```
git clone https://github.com/maralorn/tasktree
cd tasktree
cargo install
```

### Configure Taskwarrior

Add these two lines to your `.taskrc`.
```
uda.partof.type=string
uda.partof.label=Part of
```
(If you don’t do this, you will get a lot of garbage in descriptions when you try to create a tree. Otherwise, you can use the tool without this. But then you have a task list, not a tree.)

### Run


Make sure `$HOME/.cargo/bin/` is in your path.

```
tasktree
```
(You can also just hit it with `cargo run` in the repo.)

## MIT License

Copyright 2017 Malte Brandy

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
