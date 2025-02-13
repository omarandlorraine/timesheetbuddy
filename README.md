# timesheetbuddy
fills in timesheets

For reasons having to do with UK tax breaks, employers want their software guys
to fill in timesheets, a task universally hated. So I knocked this simple
program together to do that task.

The idea with this program is that when you are doing "something", you can type
`timesheetbuddy job <job> <description>` to add an entry to the timesheet. By
also keeping track of the start and end of your working day, it extrapolates
how much time you're spending on every job/description pair.

I work a 9-5 (what a way to make a living) so my crontab has the following
lines:

```
0 9  * * 1-5 timesheetbuddy start
0 17 * * 1-5 timesheetbuddy end
```

That effectively starts a day at 9 AM and ends it at 5 PM. Now between those
times, I can add events, like this example:

```
timesheetbuddy job project_x feature_a
```

and timesheetbuddy will extrapolate that the time since the previous entry in
the database (which could be a `start` of the day, or it could be another `job`
entry) has been spent on feature_a of project_x.

Obviously git could be configured to fill in `job`s with something like this in
the `post-receive` hook:

```
#!/usr/bin/bash
while read old_rev new_rev ref_name
do
        timesheetbuddy job name_of_repository $ref_name
done
```

Now, every time you push to that repository, the entry will be added to
timesheetbuddy.

Then to regenerate the timesheet every evening and serve it using apache in a
manner suitable for accountants, I do this:

```
0 18 * * 1-5 timesheetbuddy report $(date +"%Y") $(date +"%m") > /var/www/html/timesheetbuddy/$(date +"%Y-%b").csv
```

Feel free to use this for your own purposes, adapting it as necessary. I'll
welcome patches.
