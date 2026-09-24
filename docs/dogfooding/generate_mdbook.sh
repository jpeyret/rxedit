#!/usr/bin/env zsh
# this script is used to massage the files coming out of rxedit --help <topic>
# to the format need by crate mdBook.
# it takes one optional argument which is a filter to limit which help topics
# are being generated ex:  `generate_mdbook.sh 'less|more'` will only generate `less`
# and `more`
# known topics are discovered by looking for *.md files in src/help

# Actual transformations done:
# rxedit:        `<command>` by itself in the rxedit --help outputs gets transformed to a markdown [<command>](<prefix>/<command>.md)
# rxedit:         leading `rxedit ...` examples get prefixed by <br><br> linefeeds to get them to stand out more
# postformat.py:  wraps basic --help tabular output in a markdown table 
# postformat.py:  figures out how to resolve the path to the markdown files in links.  command -> .. or . -> commands/ or . sibling?


# `$verbose` can also be set to provide some diagnostic info about what it is doing

if [[ -z "$BASH_VERSION" ]]; then
    #zsh
    BASH_SOURCE=${(%):-%x}
fi

if ! command -v realpath >/dev/null 2>&1; then
    >&2 printf "\n\nERROR: required command not found: realpath\n"
    >&2 printf "Install coreutils or add a fallback resolver before running this script.\n\n"
    exit 1
fi

this_dir=$(dirname $(realpath $BASH_SOURCE))

#$1 is taken as a filter to decide which conversions to process
re_todo_filter=${1:-.}
re_todo_filter=${re_todo_filter//../.*}

dn_root=$(realpath $this_dir/../..)
dn_docs=$dn_root/docs
dn_dogfooding=$dn_docs/dogfooding
dn_mdbook=$dn_docs/src
dn_worktemp=$dn_dogfooding/temp
dn_macros=$dn_dogfooding/macros

pyformat=$dn_root/docs/dogfooding/postformat.py


dn_src_help=$(realpath $dn_root/src/help )

if [[ -n "$verbose" ]]; then

    printf '👆 %-60s 👉 %s \n' "\$this_dir" "$this_dir"
    printf '👆 %-60s 👉 %s \n' "\$dn_root" "$dn_root"
    printf '👆 %-60s 👉 %s \n' "\$dn_docs" "$dn_docs"
    printf '👆 %-60s 👉 %s \n' "\$dn_src_help" "$dn_src_help"
    printf '👆 %-60s 👉 %s \n' "\$re_todo_filter (filter markdowns to process):" "$re_todo_filter"
    printf '👆 %-60s 👉 %s \n' "\$dn_worktemp" "$dn_worktemp"
    printf '👆 %-60s 👉 %s \n' "\$dn_macros" "$dn_macros"
    printf '👆 %-40s 👉 %s \n' "\$dn_mdbook" "$dn_mdbook"

fi

if [[ ! -d $dn_root ]]; then
    >&2 printf "\n\n❌⚙️❌⚙️❌⚙️❌ \$dn_root $dn_root - project root  - does not exist or is not a directory  \n\n"
    exit 1
fi

if [[ ! -f $dn_root/Cargo.toml ]]; then
    >&2 printf "\n\n❌⚙️❌⚙️❌⚙️❌ \$dn_root $dn_root - project root  - does not have Cargo.toml in it  \n\n"
    exit 1
fi

if [[ ! -f $dn_root/Cargo.toml ]]; then
    >&2 printf "\n\n❌⚙️❌⚙️❌⚙️❌ \$dn_root $dn_root - project root  - does not have Cargo.toml in it.\n\n"
    exit 1
fi


#############################
# load the topics array from markdown files in src/help
#############################

typeset -A di_topic
di_topic=()
topic_array=( user_messages shortcodes sample README)
for pa_file in $(find $dn_src_help -name "*.md" | egrep -v "footer|header|caveat")
do

    pa_src_md=${pa_file#$dn_src_help/}
    fn=$(basename $pa_src_md)
    topic=${fn:r}
    dn_topic=${pa_src_md:h}

    di_topic[$topic]=$pa_src_md

    if [[ "$topic" == "append_prepend" ]]; then
        helptopic=''
        topic_array+=( append )
        di_topic[append]="commands/append.md"
        topic_array+=( prepend )
        di_topic[prepend]="commands/prepend.md"
        continue
    fi

    #are we matching the filter?
    if [[ ! "$topic" =~ $re_todo_filter ]]; then
        if [[ -n "$verbose" ]]; then
            printf '⏭️ skip %-60s (filter: %s)\n' "$topic" "$re_todo_filter"
        fi
        continue
    fi

    topic_array+=($pa_src_md)

done

#############################
# clear the temp directory of work files
#############################
if [[ ! -f $dn_worktemp/_imtempdir ]]; then

    # `_imtempdir` is a marker file to make sure we are on the correct directory before deleting stuff
    >&2 printf "\n\n❌⚙️❌⚙️❌⚙️❌ \$dn_worktemp $dn_worktemp/_imtempdir does not exist.\n"
    exit 1
fi
rm $dn_worktemp/*.out
rm $dn_worktemp/*.md

#############################
# call rxedit --help <topic> and dump it out for formatting
#############################

>&2 printf "\n\n🐝 dump out calls rxedit --help <topic> to {} \n\n" "$dn_worktemp"
for pa_src_md in "${topic_array[@]}"
do
    fn=$(basename $pa_src_md)
    topic=${fn:r}

    #are we matching the filter?
    if [[ ! "$topic" =~ $re_todo_filter ]]; then
        if [[ -n "$verbose" ]]; then
            printf '⏭️ skip %-60s (filter: %s)\n' "$topic" "$re_todo_filter"
        fi
        continue
    fi

    # calls `rxedit --help` by itself, and then saves it as `about.md`
    helptopic=$topic
    if [[ "$topic" == "about" ]]; then
        helptopic=''
    fi

    if [[ "$topic" == "README" ]]; then
        helptopic=''
    fi


    printf "topic \`%s\` \n" "$topic" 

    # call rxedit to show the topic help
    pa_help="$dn_worktemp/$topic.out"
    if rxedit --help $helptopic > "$pa_help"; then
        if [[ -n "$verbose" ]]; then
            printf "  ✅ rxedit --help %s" "$topic"
        fi
    else
        >&2 printf "  ❌ error running rxedit --help %s   \n\n" "$topic"
        exit 1
    fi  
    echo

done



generate_topic(){  # generate given topic

    topic=$1

    if [[ ! "$topic" =~ $re_todo_filter ]]; then
        if [[ -n "$verbose" ]]; then
            printf '⏭️ skip %-60s (filter: %s)\n' "$topic" "$re_todo_filter"
        fi
        return 0
    fi

    pa_src_md=${di_topic[$topic]}
    dn_o=${pa_src_md:h}

    pa_help="$dn_worktemp/$topic.out"


    if [[ -n "$verbose" ]]; then
        printf '👆 %-60s 👉 :%s: \n' "\$topic" "$topic"
        printf '👆 %-60s 👉 %s \n' "153\$pa_src_md" "$pa_src_md"
        printf '👆 %-40s 👉 %s \n' "\$dn_o" "$dn_o"

        printf '👆 %-60s 👉 %s \n' "\$pa_help" "$pa_help"
        printf '👆 %-60s 👉 %s \n' "\$dn_o" "$dn_o"
        # for di_topic_key di_topic_val in ${(kv)di_topic}; do
        #     printf '    🔬%-60s 👉 :%s: \n' "\$di_topic_key" "$di_topic_key"
        #     printf '    🔬%-60s 👉 :%s: \n' "\$di_topic_val" "$di_topic_val"
        # done
    fi

    comm_show="$dn_macros/_show.rxi"

    # test for a specific change macro
    macro=$dn_macros/$topic.change.rxi
    if [[ -f $macro ]]; then
        comm_change="$macro"
    else
        comm_change="$dn_macros/_change.rxi"
    fi



    #rxedit command to hide '`$topic`' lines to avoid self-referencing.
    #the topic may need custom hide which you would put in a pre-defined macro.
    macro=$dn_macros/$topic.less.rxi
    if [[ ! -f "$macro" ]]; then
        # this is a bit tricky because a "`" backtick means something both to markdown and to the shell
        comm_less1="less::\`"$topic"\`"
        if [[ -n "$verbose" ]]; then
            >&2 printf '👆 %-40s 👉 %s \n' "\$topic" "$topic"
            >&2 printf '\n🔬 %-40s 👉%s👈 \n' "\$comm_less1" "$comm_less1"
            # return
        fi
        comm_less2=
    else
        comm_less1="macro"
        comm_less2="$macro"
    fi

    #do we have a custom post-processor to run?  if not, point to a macro file that does nothing
    #as it's easier to generate a command line with `macro $somepath` that way
    comm_postchange=$dn_macros/donothing.rxi
    macro=$dn_macros/$topic.postchange.rxi
    if [[ -f $macro ]]; then
        comm_postchange="$macro"
        if [[ -n "$verbose" ]]; then
            printf '👆 %-60s 👉 %s \n' "\$comm_postchange" "$comm_postchange"
        fi
    fi

    dn_out=$dn_mdbook/$dn_o
    mkdir -p $dn_out 

    # README goes up to the root
    if [[ "$topic" == "README" ]]; then
        dn_out="$dn_mdbook/../.."
    fi


    #first pass: rxedit just writes the transformed md to the temp directory
    pa_out="$dn_worktemp/$topic.md"

    if [[ -n "$verbose" ]]; then
        echo 🔬 rxedit $pa_help $comm_show "$comm_less1" "$comm_less2" $comm_change $comm_postchange -o $pa_out 
    fi

    # one of the easier to troubleshoot this is to add an explain at the end, and `2>&1 | tee <somefile.debug>` return without further processing
    # you can also cut out parts of it to see like just `rxedit $pa_help macro $comm_show "$comm_less1"` to see where errors happen.

    if rxedit $pa_help macro $comm_show "$comm_less1" $comm_less2 macro $comm_change macro $comm_postchange -o $pa_out; then
        if [[ -n "$verbose" ]]; then
            printf "\n✅\n"
        fi
    else
        >&2 echo " ❌ error for rxedit $topic " 
        return 1
    fi

    pa_mdbook=$dn_out/$topic.md

    # pyformat.py will wrap ------ stuff in markdown table 
    # it also computes relative links to other pages.
    if python3 $pyformat $pa_out $pa_mdbook; then
        if [[ -n "$verbose" ]]; then
            printf '  ✅ pyformat %-40s 👉 %s \n' "\$pa_mdbook" "$pa_mdbook"
        fi
    else
        printf "\n❌\n py $pyformat $pa_out $pa_mdbook" 
    fi

}

printf "\n\n🐝 generating matching files...\n\n"


for pa_src_md in "${topic_array[@]}"
do
    fn=$(basename $pa_src_md)
    topic=${fn:r}

    #are we matching the filter?
    if [[ ! "$topic" =~ $re_todo_filter ]]; then
        if [[ -n "$verbose" ]]; then
            printf '⏭️ skip %-60s (filter: %s)\n' "$topic" "$re_todo_filter"
        fi
        continue
    fi

    printf "\n  generating \`%s\`\n" "$topic"
    generate_topic $topic

done
