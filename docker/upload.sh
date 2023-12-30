# binaries
DOCKER_BIN=/usr/bin/docker
TOMLQ_BIN=/usr/bin/tomlq

# script location
PWD=`pwd`
DIR=`dirname "$0"`
SCRIPT_LOCATION=$PWD/$DIR
echo $SCRIPT_LOCATION

# Package metadata 
VERSION=$(cat $SCRIPT_LOCATION/../Cargo.toml | $TOMLQ_BIN -r ".package.version")
PACKAGE_NAME=$(cat $SCRIPT_LOCATION/../Cargo.toml | $TOMLQ_BIN -r ".package.name")


#Docker metadata
DOCKER_HUB_USERNAME=mgreif
IMAGE_NAME=$DOCKER_HUB_USERNAME/$PACKAGE_NAME:$VERSION



echo "---META---"
echo "+ Version     : $VERSION"
echo "+ Package Name: $PACKAGE_NAME"
echo "+ Image Name  : $IMAGE_NAME"

echo Waiting 4 seconds for meta validation
sleep 4

echo logging into docker hub;

$DOCKER_BIN login;

if [ $? -ne 0 ];
then
    echo failed logging into docker hub;
    exit 1
fi;

echo Checking for invalid or existing image ...

if $DOCKER_BIN image ls | awk '{split($0,a," "); print a[1]":"a[2]}' | grep -q $IMAGE_NAME;
then
    echo Image with name $IMAGE_NAME is already present;
    echo Maybe you forgot to update the version?;
    exit 1
fi;

echo Building image

cd $SCRIPT_LOCATION/..

$DOCKER_BIN build -t $IMAGE_NAME -f $SCRIPT_LOCATION/Dockerfile $SCRIPT_LOCATION/..

if [ $? -ne 0 ];
then
    echo failed building image;
    exit 1
fi;

cd -

echo Pushing $IMAGE_NAME to dockerhub;

$DOCKER_BIN push $IMAGE_NAME

if [ $? -ne 0 ];
then
    echo failed pushing the image to docker_hub;
    exit 1
fi;

echo Successfully pushed to docker hub
echo Done ...

exit 0


