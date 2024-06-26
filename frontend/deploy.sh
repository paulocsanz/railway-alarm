#!/usr/bin/env bash

set -e

SCRIPTPATH="$( cd "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"

NODE_ENV='production' npm run build
if [  $? -ne 0 ]; then
  echo "Build failed"
  exit 1
fi

cp $SCRIPTPATH/dist/index.html $SCRIPTPATH/dist/404.html

rm -rf $SCRIPTPATH/../railway-alarm-nginx/site
mkdir -p $SCRIPTPATH/../railway-alarm-nginx/site

cd $SCRIPTPATH/../railway-alarm-nginx
  git init
  git config init.defaultBranch main
  git branch -m main
  git remote add origin git@github.com:paulocsanz/railway-alarm-nginx.git || echo "0"
  git fetch -p
  git checkout main
  git reset --hard origin/main

  cp -r $SCRIPTPATH/dist/** $SCRIPTPATH/../railway-alarm-nginx/site/
  echo "site-production-84f0.up.railway.app" > $SCRIPTPATH/../railway-alarm-nginx/CNAME

  git add -A
  git commit -m "Deploy"
  git push origin main -f
cd -
